mod opcodes;
mod cpu;
mod constants;
mod screen;
mod input;

use cpu::Core;
use std::fs::File;
use std::path::PathBuf;

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use cpu_simulation::constants::VRAM_START;
use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH, SCREEN_VIRTUAL_HEIGHT, SCREEN_VIRTUAL_WIDTH};
use crate::input::handle_input;

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
        SCREEN_VIRTUAL_WIDTH,
        SCREEN_VIRTUAL_HEIGHT,
        WindowOptions {
            scale_mode: ScaleMode::AspectRatioStretch,
            scale: Scale::X1,
            ..WindowOptions::default()
        },
    ).expect("Failed to create window");

    let mut buffer = vec![0u32; SCREEN_WIDTH * SCREEN_HEIGHT];

    while window.is_open() && !window.is_key_down(Key::Escape)  && cpu.running && !cpu.halted{
        // Draw one white pixel at x=32, y=15
        //buffer[15 * SCREEN_WIDTH + 32] = 0xFFFFFF;

        handle_input(&mut cpu, &mut window);

        for _ in 0..50_000 {
            if cpu.halted || !cpu.running { break; }
            cpu.step();
        }

        for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
            cpu.mem[VRAM_START + i] = i as u8 % 16;
        }

        for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
            let pixel_byte = cpu.mem[VRAM_START + i];
            let color_index = pixel_byte & 0x0F;

            //todo do something with the upper 4 bit metadata

            let final_color = screen::lookup_palette(color_index as usize);
            buffer[i] = final_color;
        }


        window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT).expect("Failed to update window");
    }
}