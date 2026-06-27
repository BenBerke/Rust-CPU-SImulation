use crate::constants::{IO_INPUT_SIZE, IO_INPUT_START};
use crate::cpu::Core;
use minifb::{Key, Window};

pub fn handle_input(cpu: &mut Core, window: &Window) {
    let tracked_keys = [
        // Letters (A-Z)
        (Key::A, 0x41), (Key::B, 0x42), (Key::C, 0x43), (Key::D, 0x44), (Key::E, 0x45),
        (Key::F, 0x46), (Key::G, 0x47), (Key::H, 0x48), (Key::I, 0x49), (Key::J, 0x4A),
        (Key::K, 0x4B), (Key::L, 0x4C), (Key::M, 0x4D), (Key::N, 0x4E), (Key::O, 0x4F),
        (Key::P, 0x50), (Key::Q, 0x51), (Key::R, 0x52), (Key::S, 0x53), (Key::T, 0x54),
        (Key::U, 0x55), (Key::V, 0x56), (Key::W, 0x57), (Key::X, 0x58), (Key::Y, 0x59),
        (Key::Z, 0x5A),
        // Numbers (0-9)
        (Key::Key0, 0x30), (Key::Key1, 0x31), (Key::Key2, 0x32), (Key::Key3, 0x33),
        (Key::Key4, 0x34), (Key::Key5, 0x35), (Key::Key6, 0x36), (Key::Key7, 0x37),
        (Key::Key8, 0x38), (Key::Key9, 0x39),
        // Special/Control
        (Key::Space, 0x20), (Key::Enter, 0x0D), (Key::Backspace, 0x08), (Key::Escape, 0x1B),
    ];

    for (key, offset) in tracked_keys.iter() {
        let is_down = window.is_key_down(*key);

        let mem_addr = IO_INPUT_START + (*offset as usize);
        if mem_addr > IO_INPUT_START + IO_INPUT_SIZE { panic!("[INPUT] Segfault. Input is beyond the Input MMIO Section"); }
        cpu.write_byte(mem_addr,if is_down { 1 } else { 0 } );
    }
}