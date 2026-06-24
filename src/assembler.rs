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
    Sym // Labels
}

fn check_op_type(token: &str, t: Operand) -> bool {
    match t {
        Operand::Reg => token.starts_with('$'),
        Operand::Imm => token.starts_with('#'),
        Operand::Sym => token.starts_with(':'),
    }
}
fn is_opcode(token: &str) -> bool {
    matches!(token, "HALT" | "LOAD" | "ADD" | "STORE" | "JMP")
}

fn get_opcode_val(token: &str) -> u64 {
    match token {
        "HALT" => OP_HALT,
        "LOAD" => OP_LOAD,
        "ADD" => OP_ADD,
        _ => OP_HALT
    }
}

// Assume format: [Opcode (16bit) | Op1 (16bit) | Op2 (16bit) | Op3 (16bit)]
fn compile_source(source_path: &str) -> Vec<u8> {
    let source_code = fs::read_to_string(source_path).expect("Failed to read source file");
    let tokens: Vec<&str> = source_code.split_whitespace().collect();

    let mut symbol_table: HashMap<String, usize> = HashMap::new();
    let mut current_offset = 0;
    let mut compiled_bytes: Vec<u8> = Vec::new();

    for token in &tokens {
        if token.starts_with(':') {
            let label = token.trim_start_matches(':');
            symbol_table.insert(label.to_string(), current_offset);
        }
        else if is_opcode(token) { current_offset += 8; }
    }

    // Does not execute logic. Turns the text file into an execute ready bytes and checks for bugs

    let mut line_count: usize = 0;
    let mut i: usize = 0;
    while i < tokens.len() {
        let opcode = tokens[i];

        if opcode.starts_with(";") { line_count += 1; continue;}
        if !is_opcode(opcode) { i += 1; continue; }

        let arg1 = tokens[i + 1];
        let arg2 = tokens[i + 2];
        let arg3 = tokens[i + 3];

        let mut instr: u64 = 0;

        let number_arg1 = arg1.trim_start_matches(|c| c == '$' || c == '#' || c == ':');
        let value1: u64 = number_arg1.parse::<u64>().expect("Failed to parse operand number");
        let number_arg2 = arg2.trim_start_matches(|c| c == '$' || c == '#' || c == ':');
        let value2: u64 = number_arg2.parse::<u64>().expect("Failed to parse operand number");
        let number_arg3 = arg3.trim_start_matches(|c| c == '$' || c == '#' || c == ':');
        let value3: u64 = number_arg3.parse::<u64>().expect("Failed to parse operand number");

        instr |= get_opcode_val(opcode) | (value1 << 16) | (value2 << 32) | (value3 << 48);

        let bytes = instr.to_le_bytes();
        compiled_bytes.extend_from_slice(&bytes);

        match opcode{
            ".ADD" => {
                if check_op_type(opcode, Operand::Reg) && check_op_type(arg1, Operand::Reg) && check_op_type(arg2, Operand::Reg) {
                    i += 1;
                    continue;
                }

                println!("Error on line: {}", line_count);
                break;
            }
            _=> {i += 1; continue;}
        }
    }


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