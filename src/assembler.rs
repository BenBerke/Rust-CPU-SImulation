use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use cpu_simulation::opcodes::*;
use cpu_simulation::constants::SIZE_SECTOR;


fn assemble_to_disk(source_path: String, disk_file: &mut File, sector_number: u64) {
    let source_code = fs::read_to_string(&source_path).expect("Failed to read source file");
    let mut compiled_bytes: Vec<u8> = Vec::new();

    let tokens: Vec<&str> = source_code.split_whitespace().collect();
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

    // Ensure it fits within one sector (512 bytes)
    if compiled_bytes.len() > SIZE_SECTOR as usize { panic!("[ASSEMBLER] Error: File too large for one sector!"); }

    // Pad with zeros to reach 512 bytes
    while compiled_bytes.len() < SIZE_SECTOR as usize { compiled_bytes.push(0); }

    // Write to the specific sector offset
    let disk_offset = sector_number * SIZE_SECTOR;
    disk_file.seek(SeekFrom::Start(disk_offset)).expect("Seek failed");
    disk_file.write_all(&compiled_bytes).expect("Write failed");

    println!("[ASSEMBLER] Successfully wrote to sector {}", sector_number);
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

    // 2. Assemble the Kernel into Sector 1
    assemble_to_disk(
        root_dir.join("os").join("kernel").to_string_lossy().into_owned(),
        &mut disk_file,
        1
    );

    println!("[ASSEMBLER] Disk image built with Bootloader and Kernel.");
}