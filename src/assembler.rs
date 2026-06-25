use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use cpu_simulation::opcodes::*;
use cpu_simulation::constants::SIZE_SECTOR;
use crate::Operand::{Imm32, Reg};

#[derive(Debug, PartialEq)]
enum Operand{
    Reg,
    Imm,
    Imm32,
    Sym, // Labels
    PH, // Padding
}

struct SourceLine {
    line_number: usize,
    tokens: Vec<String>
}

fn check_op_type(token: &str, t: Operand) -> bool {
    match t {
        Operand::Reg => token.starts_with('$'),
        Operand::Imm => token.starts_with('#'),
        Operand::Imm32 => token.starts_with('%'),
        Operand::Sym => token.starts_with(':'),
        Operand::PH => token.starts_with('@'),
    }
}
fn get_opcode_val(token: &str) -> u64 {
    let Some(stripped) = token.strip_prefix('.') else { return !0; };
    Opcode::from_str(stripped).unwrap_or(!0)
}

fn parse_number(clean: &str)->Result<u64, String>{
    if clean.is_empty() { return Err("empty literal".to_string()); }

    if let Some(hex) = clean.strip_prefix("0x").or_else(|| clean.strip_prefix("0X")) {
        return u64::from_str_radix(hex, 16)
            .map_err(|_| format!("invalid hex literal '{}'", clean));
    }

    clean.parse::<u64>().map_err(|_| format!("invalid decimal literal '{}'", clean))
}

// Assume format: [Opcode (16bit) | Op1 (16bit) | Op2 (16bit) | Op3 (16bit)]
fn compile_source(source_path: &str) -> Vec<u8> {
    use Operand::*;

    let source_code = fs::read_to_string(source_path).expect("Failed to read source file");
    let mut chars = source_code.chars().peekable();

    let mut symbol_table: HashMap<String, usize> = HashMap::new();
    let mut current_offset = 0;

    let mut compiled_bytes: Vec<u8> = Vec::new();
    let mut line_count: usize = 1;

    let mut program_lines: Vec<SourceLine> = Vec::new();
    let mut current_line_tokens = Vec::new();

    while chars.peek().is_some() {
        let c = *chars.peek().unwrap();

        if c == '\n' {
            if !current_line_tokens.is_empty() {
                program_lines.push(SourceLine {line_number: line_count, tokens: current_line_tokens});
            }
            current_line_tokens = Vec::new();

            line_count += 1;
            chars.next();
            continue;
        }
        if c.is_whitespace() {chars.next(); continue; }
        if c == ';' {
            while let Some(&next_c) = chars.peek() {
                if next_c == '\n' { break;}
                chars.next();
            }
            continue;
        }

        let mut current_word = String::new();
        while let Some(&next_c) = chars.peek() {
            if next_c.is_whitespace() || next_c == ';' { break; }
            current_word.push(chars.next().unwrap());
        }

        current_line_tokens.push(current_word);
    }

    // Catch any remaining tokens if the file didn't end with a newline
    if !current_line_tokens.is_empty() {
        program_lines.push(SourceLine {line_number: line_count, tokens: current_line_tokens});
    }

    // PASS 1 (Symbol Table Generation)
    for line in &program_lines {
        let first_token = &line.tokens[0];

        if first_token.starts_with(':') {
            let label = first_token.trim_start_matches(':');
            symbol_table.insert(label.to_string(), current_offset);
        }

        if get_opcode_val(first_token) != !0 { current_offset += 8; }
    }

    let mut standard_compilation_success = true;
    // PASS 2 (Byte Compilation)
    // Does not execute logic. Turns the text file into an execute ready bytes and checks for bugs
    for mut line in program_lines {
        // Pad the tokens vector so it ALWAYS has at least 4 elements (Opcode + 3 Args)
        // Uuse "@0" as a placeholder literal
        if line.tokens.len() == 1 && line.tokens[0].starts_with(':') { continue; }
        while line.tokens.len() < 4 { line.tokens.push("@0".to_string()); }

        let opcode = &line.tokens[0];
        let opcode_val = get_opcode_val(opcode);

        if opcode_val == !0 {
            println!("[COMPILER ERROR] Unknown instruction '{}' on line: {}", opcode, line.line_number);
            standard_compilation_success = false;
            break;
        }

        let arg1 = &line.tokens[1];
        let arg2 = &line.tokens[2];
        let arg3 = &line.tokens[3];

        use Opcode::*; // Import all variants

        let is_valid = match Opcode::try_from(opcode_val as u16) {
            Ok(Halt) => true,
            Ok(LoadImm) => check_op_type(arg1, Reg) && check_op_type(arg2, Imm),
            Ok(Add) | Ok(Sub) | Ok(Mul) | Ok(Div) | Ok(SaveDisk) => {
                check_op_type(arg1, Reg) && check_op_type(arg2, Reg) && check_op_type(arg3, Reg)
            },
            Ok(LoadMem) => check_op_type(arg1, Reg) && check_op_type(arg2, Imm32),
            Ok(Jmp) => symbol_table.contains_key(arg1.trim_start_matches(':')) || check_op_type(arg1, Imm),
            Ok(JmpAbs) => check_op_type(arg1, Imm32),
            Ok(JumpZero) => check_op_type(arg1, Sym) && check_op_type(arg2, Reg),
            Ok(Store) => check_op_type(arg1, Imm32) && check_op_type(arg2, Reg),
            Err(_) => false,
        };

        if !is_valid {
            println!("[COMPILER ERROR] Invalid arguments for '{}' on line: {}", opcode, line.line_number);
            standard_compilation_success = false;
            break;
        }

        let resolve_arg = |token: &str| -> Result<u64, String> {
            let clean_token = token.trim_start_matches(|c| c == '$' || c == '#' || c == ':' || c == '@' || c == '%');

            if let Some(&address) = symbol_table.get(clean_token) { Ok(address as u64) }
            else { parse_number(clean_token) }
        };

        let mut val1 = match resolve_arg(arg1) {
            Ok(v) => v,
            Err(e) => {
                println!(
                    "[COMPILER ERROR] Failed to parse arg1 '{}' on line {}: {}",
                    arg1,
                    line.line_number,
                    e
                );
                standard_compilation_success = false;
                break;
            }
        };

        let mut val2 = match resolve_arg(arg2) {
            Ok(v) => v,
            Err(e) => {
                println!(
                    "[COMPILER ERROR] Failed to parse arg2 '{}' on line {}: {}",
                    arg2,
                    line.line_number,
                    e
                );
                standard_compilation_success = false;
                break;
            }
        };

        let mut val3 = match resolve_arg(arg3) {
            Ok(v) => v,
            Err(e) => {
                println!(
                    "[COMPILER ERROR] Failed to parse arg3 '{}' on line {}: {}",
                    arg3,
                    line.line_number,
                    e
                );
                standard_compilation_success = false;
                break;
            }
        };

        // Special case for some instruction where first number is 32 bit.
        let mut instr: u64 = 0;
        if let Ok(Opcode::Store) = Opcode::try_from(opcode_val as u16) {
            let full_address = val1; // Hold the 32-bit address (1)
            let reg_source = val2;   // Hold the register index (1)

            val1 = (full_address >> 16) & 0xFFFF;
            val2 = full_address & 0xFFFF;
            val3 = reg_source;

            instr = opcode_val | (val1 << 16) | (val2 << 32) | (val3 << 48);
        } else { instr |= opcode_val | (val1 << 16) | (val2 << 32) | (val3 << 48); }

        // --- DETAILED ASSEMBLER DEBUGGER ---
        #[cfg(debug_assertions)]
        {
            println!(
                "LINE {:03} | {:<6} -> RAW BINARY: [OP: 0x{:04X} | VAL1: 0x{:04X} | VAL2: 0x{:04X} | VAL3: 0x{:04X}] -> TOTAL: 0x{:016X}",
                line.line_number,
                opcode,
                opcode_val,
                val1,
                val2,
                val3,
                instr
            );
        }

        compiled_bytes.extend_from_slice(&instr.to_le_bytes());
    }

    if !standard_compilation_success { compiled_bytes.clear(); }
    compiled_bytes
}

fn assemble_to_disk(source_path: String, disk_file: &mut File, sector_number: u64) {
    let mut bytes = compile_source(&source_path);

    if bytes.len() > (SIZE_SECTOR - 2) as usize { panic!("File too large!");  }
    while bytes.len() < (SIZE_SECTOR - 2) as usize { bytes.push(0); }

    // Magic values to tell the CPU that the bootloader is loaded
    bytes.push(0x55);
    bytes.push(0xAA);

    let disk_offset = sector_number * SIZE_SECTOR;
    disk_file.seek(SeekFrom::Start(disk_offset)).expect("Seek failed");
    disk_file.write_all(&bytes).expect("Write failed");
}

fn assemble_to_disk_multisector(source_path: String, disk_file: &mut File, start_sector: u64){
    let raw_code_bytes = compile_source(&source_path);
    let mut bytes = Vec::new();

    bytes.push(0x44);
    bytes.push(0xBB);

    bytes.extend(raw_code_bytes);

    let remainder = bytes.len() % SIZE_SECTOR as usize;
    if remainder != 0 { for _ in 0..(SIZE_SECTOR as usize - remainder) { bytes.push(0); } }

    let chunks = bytes.chunks(SIZE_SECTOR as usize);
    for (i, chunks) in chunks.enumerate() {
        let sector_number = start_sector + i as u64;
        let disk_offset = sector_number * SIZE_SECTOR;

        disk_file.seek(SeekFrom::Start(disk_offset)).expect("Seek failed");
        disk_file.write_all(&chunks).expect("Write failed");

        println!("[ASSEMBLER] Written chunk {} to sector {}", i, sector_number);
    }
}

fn main(){
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut disk_file = OpenOptions::new()
        .read(true).write(true).create(true)
        .open(root_dir.join("memory").join("disk_storage"))
        .expect("Couldn't open disk file");

    // 1. Assemble the Bootloader into Sector 0
    assemble_to_disk(
        root_dir.join("os").join("boot_loader").to_string_lossy().into_owned(),
        &mut disk_file,
        0
    );

    // 2. Assemble the Kernel starting from Sector 1
    assemble_to_disk_multisector(
        root_dir.join("os").join("kernel").to_string_lossy().into_owned(),
        &mut disk_file,
        1
    );

    println!("[ASSEMBLER] Disk image built with Bootloader and Kernel.");
}