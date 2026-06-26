use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use cpu_simulation::opcodes::*;
use cpu_simulation::constants::{BOOTLOADER_BASE_ADDRESS, DATA_START, KERNEL_CODE_ADDRESS, SECTION_DATA, SECTION_TEXT, SIZE_SECTOR};
use crate::Operand::{Imm32, Reg};

#[derive(Debug, PartialEq)]
enum Operand{
    Reg,
    Imm,
    Imm32,
    Sym, // Labels
    PH, // Padding
    Data,
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
        Operand::PH => token.starts_with('|'),
        Operand::Data => token.starts_with('!'),
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
fn compile_source(source_path: &str, use_data_layout: bool, base_address: usize) -> Vec<u8> {
    use Operand::*;

    let source_code = fs::read_to_string(source_path).expect("Failed to read source file");
    let mut chars = source_code.chars().peekable();

    let mut symbol_table: HashMap<String, usize> = HashMap::new();
    let mut current_offset = base_address;
    let mut current_data_address = base_address + DATA_START;

    let mut compiled_bytes: Vec<u8> = Vec::new();
    let mut data_bytes: Vec<u8> = Vec::new();
    let mut line_count: usize = 1;

    let mut program_lines: Vec<SourceLine> = Vec::new();
    let mut current_line_tokens = Vec::new();

    let mut current_section = SECTION_TEXT.to_string();

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
    if !current_line_tokens.is_empty() { program_lines.push(SourceLine {line_number: line_count, tokens: current_line_tokens}); }

    // PASS 1 (Symbol Table Generation)
    for line in &program_lines {
        let first_token = &line.tokens[0];

        // clean_label is exactly first_token without the '@'
        if let Some(label) = first_token.strip_prefix('@') { current_section = label.to_string(); continue; }

        if current_section == SECTION_DATA{
            if let Some(label) = first_token.strip_prefix('!') {
                let data_type = &line.tokens[1];
                symbol_table.insert(label.to_string(), current_data_address);

                let allocation_size = match data_type.as_str() {
                    "db" => 1,
                    "dw" => 2,
                    "dd" => 4,
                    _=>1,
                };

                current_data_address += allocation_size;
            }

            continue;
        }

        if current_section == SECTION_TEXT{
            if let Some(label) = first_token.strip_prefix(':') { symbol_table.insert(label.to_string(), current_offset); }
            if get_opcode_val(first_token) != !0 { current_offset += 8; }
        }
    }

    let mut standard_compilation_success = true;
    // PASS 2 (Byte Compilation)
    // Does not execute logic. Turns the text file into an execute ready bytes and checks for bugs
    for mut line in program_lines {
        if line.tokens.len() == 1 && line.tokens[0].starts_with(':') { continue; }

        let first_token = &line.tokens[0];

        if let Some(label) = first_token.strip_prefix('@') { current_section = label.to_string(); continue; }

        if current_section == SECTION_DATA{
            let name = first_token.strip_prefix('!').unwrap_or_else(|| panic!("Data must start with a '!' prefix. Line: {}", line.line_number));
            let data_type = &line.tokens[1];
            let data_val = &line.tokens[2];

            let clean_val = parse_number(data_val.trim_start_matches(|c| c == '#' || c == '%')).unwrap_or_else(|_| {
                panic!("[COMPILER ERROR] Invalid data value at line: {}", line.line_number)
            });

            match data_type.as_str() {
                "db" => data_bytes.push(clean_val as u8),
                "dw" => data_bytes.extend_from_slice(&(clean_val as u16).to_le_bytes()),
                "dd" => data_bytes.extend_from_slice(&(clean_val as u32).to_le_bytes()),
                _ => {}
            }

            continue;
        }

        // === TEXT ===

        // Pad the tokens vector so it ALWAYS has at least 4 elements (Opcode + 3 Args)
        // Uuse "|0" as a placeholder literal
        while line.tokens.len() < 4 { line.tokens.push("|0".to_string()); }

        let opcode = &line.tokens[0];
        let opcode_val = get_opcode_val(opcode);

        if opcode_val == !0 {
            println!(
                "{}[COMPILER ERROR]{} Unknown instruction '{}' on line: {}",
                "\x1b[31m", "\x1b[0m", opcode, line.line_number
            );
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
            Ok(LoadMem) => check_op_type(arg1, Reg) && (check_op_type(arg2, Imm32) || check_op_type(arg2, Data)),
            Ok(Jmp) => symbol_table.contains_key(arg1.trim_start_matches(':')) || check_op_type(arg1, Imm),
            Ok(JmpAbs) => check_op_type(arg1, Imm32),
            Ok(JumpZero) => check_op_type(arg1, Sym) && check_op_type(arg2, Reg),
            Ok(Store) => check_op_type(arg1, Imm32) && check_op_type(arg2, Reg),
            Ok(DTM) => check_op_type(arg1, Imm32) && check_op_type(arg2, Imm32) && check_op_type(arg3, Reg),
            Err(_) => false,
        };

        if !is_valid {
            println!("\x1b[1;31m[COMPILER ERROR]\x1b[0m Invalid arguments for '{}' on line: {}", opcode, line.line_number);
            standard_compilation_success = false;
            break;
        }

        let resolve_arg = |token: &str| -> Result<u64, String> {
            let clean_token = token.trim_start_matches
            (|c| c == '$' || c == '#' || c == ':' || c == '@' || c == '%' || c == ':' || c == '|' || c == '!');

            if let Some(&address) = symbol_table.get(clean_token) { Ok(address as u64) }
            else { parse_number(clean_token) }
        };

        let mut val1 = match resolve_arg(arg1) {
            Ok(v) => v,
            Err(e) => {
                println!(
                    "\x1b[1;31m[COMPILER ERROR]\x1b[0m Failed to parse arg1 '{}' on line {}: {}",
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
                    "\x1b[1;31m[COMPILER ERROR]\x1b[0m Failed to parse arg2 '{}' on line {}: {}",
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
                    "\x1b[1;31m[COMPILER ERROR]\x1b[0m Failed to parse arg3 '{}' on line {}: {}",
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

    if !standard_compilation_success { compiled_bytes.clear(); std::process::exit(1);}
    if use_data_layout {
        while compiled_bytes.len() < DATA_START {
            compiled_bytes.push(0);
        }

        compiled_bytes.extend(data_bytes);
    }
    compiled_bytes
}

fn assemble_bootloader(source_path: String, disk_file: &mut File, sector_number: u64) {
    let mut bytes = compile_source(&source_path, false, BOOTLOADER_BASE_ADDRESS);

    if bytes.len() > (SIZE_SECTOR - 2) as usize { panic!("File too large!");  }
    while bytes.len() < (SIZE_SECTOR - 2) as usize { bytes.push(0); }

    // Magic values to tell the CPU that the bootloader is loaded
    bytes.push(0x55);
    bytes.push(0xAA);

    let disk_offset = sector_number * SIZE_SECTOR;
    disk_file.seek(SeekFrom::Start(disk_offset)).expect("Seek failed");
    disk_file.write_all(&bytes).expect("Write failed");
}

fn assemble_kernel(source_path: String, disk_file: &mut File, start_sector: u64){
    let raw_code_bytes = compile_source(&source_path, true, KERNEL_CODE_ADDRESS);
    let mut bytes = Vec::new();

    // Bytes 512 & 513: Magic Kernel Signature
    bytes.push(0x44);
    bytes.push(0xBB);

    let total_unpadded_len = 2 + 2 + raw_code_bytes.len();
    let sector_count = (total_unpadded_len + SIZE_SECTOR as usize - 1) / SIZE_SECTOR as usize;

    // Bytes 514 & 515: Sector Count (Stored as a 16-bit Little-Endian value)
    let count_u16 = sector_count as u16;
    bytes.push((count_u16 & 0xFF) as u8);        // Byte 514 (Low Byte)
    bytes.push(((count_u16 >> 8) & 0xFF) as u8);  // Byte 515 (High Byte)

    // Byte 516+: Actual Kernel Code begins here
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
    assemble_bootloader(
        root_dir.join("os").join("boot_loader").to_string_lossy().into_owned(),
        &mut disk_file,
        0
    );

    // 2. Assemble the Kernel starting from Sector 1
    assemble_kernel(
        root_dir.join("os").join("kernel").to_string_lossy().into_owned(),
        &mut disk_file,
        1
    );

    println!("[ASSEMBLER] Disk image built with Bootloader and Kernel.");
}