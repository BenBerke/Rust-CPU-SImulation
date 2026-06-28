use std::fs::File;
use std::path::PathBuf;

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

use cpu_simulation::constants::{
    SCREEN_HEIGHT,
    SCREEN_WIDTH,
    SCREEN_VIRTUAL_HEIGHT,
    SCREEN_VIRTUAL_WIDTH,
};

use cpu_simulation::hardware::motherboard::Motherboard;

fn main() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let disk_storage_path = root_dir.join("memory").join("disk_storage");

    let disk_file = File::open(disk_storage_path).expect("Disk image not found! Run 'assembler' first.");

    let mut motherboard = Motherboard::new(disk_file);

    if !motherboard.launch_bios(0) { println!("[SYSTEM] Failed to launch BIOS"); return; }

    let mut window = Window::new(
        "Tilky VM",
        SCREEN_VIRTUAL_WIDTH,
        SCREEN_VIRTUAL_HEIGHT,
        WindowOptions {
            scale_mode: ScaleMode::AspectRatioStretch,
            scale: Scale::X1,
            ..WindowOptions::default()
        },
    ).expect("Failed to create window");

    while window.is_open() && !window.is_key_down(Key::Escape)  && motherboard.is_alive() {
        motherboard.handle_input(&window);

        motherboard.step_frame();

        // Temporary test pattern.
        motherboard.debug_fill_vram();

        motherboard.render_vram();

        window.update_with_buffer(&motherboard.framebuffer, SCREEN_WIDTH, SCREEN_HEIGHT).expect("Failed to update window");
    }
}