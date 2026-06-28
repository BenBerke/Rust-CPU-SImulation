use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::constants::*;
use crate::hardware::timer::Timer;

pub struct Bus {
    pub mem: [u8; SIZE_MEMORY as usize],
    pub disk_drive: File,
    pub timer: Timer,
}

impl Bus {
    pub fn new(disk_drive: File) -> Self {
        Self {
            mem: [0; SIZE_MEMORY as usize],
            disk_drive,
            timer: Timer::new(),
        }
    }

    pub fn tick_devices(&mut self, cycles: u64) {
        self.timer.tick(cycles);
    }

    pub fn read_byte(&self, addr: usize) -> u8 {
        if addr >= IO_TIMER_START && addr < IO_TIMER_START + IO_TIMER_SIZE {
            let offset = addr - IO_TIMER_START;
            return self.timer.read_byte(offset);
        }

        if addr >= SIZE_MEMORY as usize {
            println!("[BUS] Out-of-bounds read at 0x{:05X}", addr);
            return 0;
        }

        self.mem[addr]
    }

    pub fn write_byte(&mut self, addr: usize, value: u8) {
        if addr >= IO_TIMER_START && addr < IO_TIMER_START + IO_TIMER_SIZE {
            println!("[BUS] Ignored write to read-only timer at 0x{:05X}", addr);
            return;
        }

        if addr >= SIZE_MEMORY as usize {
            println!("[BUS] Out-of-bounds write at 0x{:05X}", addr);
            return;
        }

        self.mem[addr] = value;
    }

    pub fn load_sector_from_disk(
        &mut self,
        sector_number: u64,
        ram_target_address: usize,
    ) -> bool {
        let disk_offset = sector_number * SIZE_SECTOR;

        if let Err(err) = self.disk_drive.seek(SeekFrom::Start(disk_offset)) {
            println!("[DISK ERROR] Seek failed: {}", err);
            return false;
        }

        let start = ram_target_address;
        let end = start + SIZE_SECTOR as usize;

        if end > SIZE_MEMORY as usize {
            println!(
                "[DISK ERROR] Sector load target out of RAM bounds: {}..{}",
                start,
                end
            );
            return false;
        }

        if let Err(err) = self.disk_drive.read_exact(&mut self.mem[start..end]) {
            println!(
                "[DISK ERROR] Failed to read sector {}: {}",
                sector_number,
                err
            );
            return false;
        }

        true
    }

    pub fn write_to_disk(
        &mut self,
        ram_start: usize,
        ram_end: usize,
        sector: u64,
    ) -> bool {
        if ram_start > ram_end || ram_end > SIZE_MEMORY as usize {
            println!(
                "[DISK ERROR] Invalid RAM range for disk write: {}..{}",
                ram_start,
                ram_end
            );
            return false;
        }

        let disk_offset = sector * SIZE_SECTOR;

        if let Err(err) = self.disk_drive.seek(SeekFrom::Start(disk_offset)) {
            println!("[DISK ERROR] Seek failed: {}", err);
            return false;
        }

        if let Err(err) = self.disk_drive.write_all(&self.mem[ram_start..ram_end]) {
            println!("[DISK ERROR] Write failed: {}", err);
            return false;
        }

        true
    }
}