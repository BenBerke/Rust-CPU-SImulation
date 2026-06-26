mod opcodes;
mod cpu;
mod constants;

use cpu::Core;
use std::fs::File;
use std::path::PathBuf;

use minifb::{Key, Window, WindowOptions};
use cpu_simulation::constants::VRAM_START;
use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};

fn main() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let disk_storage_path = root_dir.join("memory").join("disk_storage");

    // Open the pre-built disk image
    let disk_file = File::open(disk_storage_path).expect("Disk image not found! Run 'assembler' first.");

    let mut cpu = Core::new(disk_file);

    // BIOS doesn't care how the disk was made, just that it exists
    if !cpu.launch_bios(0) {
        println!("[SYSTEM] Failed to launch BIOS");
        return;
    }

    cpu.running = true;

    let mut window = Window::new(
        "Tilky OS",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    ).expect("Failed to create window");

    let mut buffer = vec![0u32; SCREEN_WIDTH * SCREEN_HEIGHT];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Draw one white pixel at x=32, y=15
        //buffer[15 * SCREEN_WIDTH + 32] = 0xFFFFFF;

        for _ in 0..50_000 {
            if cpu.halted || !cpu.running { break; }
            cpu.step();
        }

        for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
            let color_index = cpu.mem[VRAM_START + i];
            buffer[i] = 1500; //todo add a colour palette
            //buffer[i] = lookup_palette(color_index);
        }


        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .expect("Failed to update window");
    }
}