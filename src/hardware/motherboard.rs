use std::fs::File;

use minifb::Window;

use crate::constants::{CYCLES_PER_FRAME, SCREEN_HEIGHT, SCREEN_WIDTH, VRAM_SIZE, VRAM_START};

use crate::hardware::bus::Bus;
use crate::hardware::cpu::Core;
use crate::hardware::screen;
use crate::input::handle_input;

use std::time::Instant;

pub struct Motherboard {
    pub cpu: Core,
    pub bus: Bus,

    pub framebuffer: Vec<u32>,
}

impl Motherboard {
    pub fn new(disk_drive: File) -> Self {
        Self {
            cpu: Core::new(),
            bus: Bus::new(disk_drive),

            framebuffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    pub fn launch_bios(&mut self, boot_sector: u64) -> bool {
        println!("[BIOS] Launching BIOS...");

        if !self.bus.load_sector_from_disk(boot_sector, 0)
        {
            println!("[BIOS] Failed to load boot sector");
            return false;
        }

        let sig_low = self.bus.mem[510];
        let sig_high = self.bus.mem[511];

        if sig_low == 0x55 && sig_high == 0xAA {
            println!("[BIOS] BIOS is running");

            self.cpu.pc = 0x0;
            self.cpu.running = true;
            self.cpu.halted = false;

            return true;
        }

        println!("[BIOS] No bootable device is found");
        false
    }

    pub fn is_alive(&self) -> bool { self.cpu.running && !self.cpu.halted }

    pub fn handle_input(&mut self, window: &Window) { handle_input(&mut self.bus, window); }

    pub fn step_frame(&mut self) {
        for _ in 0..CYCLES_PER_FRAME {
            if self.cpu.halted || !self.cpu.running {
                break;
            }

            self.cpu.step(&mut self.bus);
        }
    }

    pub fn render_vram(&mut self) {
        for i in 0..VRAM_SIZE {
            let pixel_byte = self.bus.mem[VRAM_START + i];
            let color_index = pixel_byte & 0x0F;

            self.framebuffer[i] = screen::lookup_palette(color_index as usize);
        }
    }

    pub fn debug_fill_vram(&mut self) {
        for i in 0..VRAM_SIZE {
            self.bus.mem[VRAM_START + i] = i as u8 % 16;
        }
    }
}