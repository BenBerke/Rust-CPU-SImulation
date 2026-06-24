mod opcodes;
mod cpu;
mod constants;

use cpu::Core;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let disk_storage_path = root_dir.join("memory").join("disk_storage");

    // Open the pre-built disk image
    let disk_file = File::open(disk_storage_path).expect("Disk image not found! Run 'assembler' first.");

    let mut cpu = Core::new(disk_file);

    // BIOS doesn't care how the disk was made, just that it exists
    if cpu.launch_bios(0) { cpu.run(); }
    else { println!("[SYSTEM] Failed to launch BIOS"); }
}