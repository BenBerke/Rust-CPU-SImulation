use std::fs;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use crate::opcodes::*;
use crate::constants::SIZE_SECTOR;

// Assemble the  bootloader and load into storage
pub fn load_bootloader(bootloader_path: String, disk_file: &mut File) {
    let mut bootloader_code = fs::read_to_string(&bootloader_path).expect("Couldn't read bootloader code");
    let mut compiled_bytes: Vec<u8> = Vec::new();

    let mut write_address = 0x7C00;

    // Parser
    let tokens: Vec<&str> = bootloader_code.split_whitespace().collect();
    for token in tokens {
        match token {
            "HALT" => compiled_bytes.push(OP_HALT),
            "LOAD" => compiled_bytes.push(OP_LOAD),
            "ADD" => compiled_bytes.push(OP_ADD),
            "STORE" => compiled_bytes.push(OP_STORE),
            val if val.chars().all(|c| c.is_numeric()) => {
                compiled_bytes.push(val.parse::<u8>().expect("Invalid number"));
            }
            val => println!("[ASSEMBLER] Warning: Skipping unknown token '{}'", val),
        }
    }

    if compiled_bytes.last() != Some(&OP_HALT) { compiled_bytes.push(OP_HALT); }

    // Padding to sector size
    while compiled_bytes.len() < (SIZE_SECTOR - 2) as usize { compiled_bytes.push(0); }

    // Magic numbers for the BIOS to know the bootloader location
    compiled_bytes.push(0x55);
    compiled_bytes.push(0xAA);

    // Write to disk
    disk_file.seek(SeekFrom::Start((0) as u64)).expect("Seek failed");
    disk_file.write_all(&compiled_bytes).expect("Write failed");

    println!("[ASSEMBLER] Bootloader loaded");
}


