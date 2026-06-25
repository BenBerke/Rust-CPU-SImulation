use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use cpu_simulation::opcodes::*;
use cpu_simulation::constants::SIZE_SECTOR;
use crate::Operand::Reg;

#[derive(Debug, PartialEq)]
enum Operand{
    Reg,
    Imm,
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
        Operand::Sym => token.starts_with(':'),
        Operand::PH => token.starts_with('@'),
    }
}
fn get_opcode_val(token: &str) -> u64 {
    let Some(stripped) = token.strip_prefix('.') else { return !0; };
    match stripped {
        "HALT" => OP_HALT,
        "LOAD" => OP_LOAD,
        "ADD" => OP_ADD,
        "JMP" => OP_JMP,
        _ => !0
    }
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

        if get_opcode_val(first_token) != !0 {
            current_offset += 8;
        }
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

        let is_valid = match opcode.strip_prefix('.').unwrap_or(opcode) {
            "ADD" | "SUB" => check_op_type(arg1, Reg) && check_op_type(arg2, Reg) && check_op_type(arg3, Reg),
            "MOV"         => check_op_type(arg1, Reg) && check_op_type(arg2, Imm),
            "JMP"         => symbol_table.contains_key(arg1), // JMP expects a Label argument
            "HALT"        => true,
            _             => false,
        };

        if !is_valid {
            println!("[COMPILER ERROR] Invalid arguments for '{}' on line: {}", opcode, line.line_number);
            standard_compilation_success = false;
            break;
        }

        let resolve_arg = |token: &str| -> u64{
            let clean_token = token.trim_start_matches(|c| c == '$' || c == '#' || c == ':' || c == '@');

            if let Some(&address) = symbol_table.get(clean_token) { address as u64 }
            else { clean_token.parse::<u64>().unwrap_or(0) }
        };

        let val1 = resolve_arg(arg1);
        let val2 = resolve_arg(arg2);
        let val3 = resolve_arg(arg3);


        let mut instr: u64 = 0;
        instr |= get_opcode_val(&*opcode) | (val1 << 16) | (val2 << 32) | (val3 << 48);

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

    if bytes.len() > SIZE_SECTOR as usize { panic!("File too large!"); }
    while bytes.len() < SIZE_SECTOR as usize { bytes.push(0); }

    let disk_offset = sector_number * SIZE_SECTOR;
    disk_file.seek(SeekFrom::Start(disk_offset)).expect("Seek failed");
    disk_file.write_all(&bytes).expect("Write failed");
}

fn assemble_to_disk_multisector(source_path: String, disk_file: &mut File, start_sector: u64){
    let mut bytes = compile_source(&source_path);

    let remainder = bytes.len() % SIZE_SECTOR as usize;
    if remainder != 0 {
        for _ in 0..(SIZE_SECTOR as usize - remainder) { bytes.push(0); }
    }

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