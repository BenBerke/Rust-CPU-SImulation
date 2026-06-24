use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

mod opcodes;
mod cpu;
mod bootloader;

use cpu::Core;
use crate::bootloader::load_bootloader;

fn main() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let bootloader_path = root_dir.join("os").join("boot_loader");
    let disk_storage_path = root_dir.join("memory").join("disk_storage");

    let disk_file = OpenOptions::new().read(true).write(true).create(true).open(disk_storage_path).expect("Couldn't open disk");

    let cpu = Core::new(disk_file);
    load_bootloader(&cpu, bootloader_path.to_string_lossy().into_owned());
}
