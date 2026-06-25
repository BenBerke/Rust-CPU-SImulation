use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::opcodes::*;
use crate::constants::*;

pub struct Core{
    registers: [u16; REG_COUNT as usize],
    memory: [u8;SIZE_MEMORY as usize], // Main memory
    disk_drive: File,

    pc: usize, // Program Counter (points to current instruction)
    running: bool
}

impl Core{
    pub fn new(disk_file: File) -> Box<Core>
    { Box::new(Core { registers: [0;REG_COUNT as usize], memory:[0;SIZE_MEMORY as usize], disk_drive: disk_file, pc: 0x7C00, running: false }) }

    pub fn consume_byte(&mut self) -> u8 {
        let byte = self.memory[self.pc];
        self.pc += 1;
        byte
    }

    pub fn consume_u16(&mut self) -> u16 {
        let low = self.consume_byte() as u16;
        let high = self.consume_byte() as u16;
        low | (high << 8)
    }

    pub fn consume_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.memory[self.pc..self.pc+8]);
        self.pc += 8;
        u64::from_le_bytes(bytes)
    }

    pub fn load_sector_from_disk(&mut self, sector_number: u64, ram_target_address: u64){
        let disk_offset = sector_number * SIZE_SECTOR;

        println!("[HARDWARE] Disk Controller: Seeking to byte offset {}...", disk_offset);
        self.disk_drive.seek(SeekFrom::Start(disk_offset));

        println!("[HARDWARE] Disk Controller: Streaming 512 bytes into RAM at address {}...", ram_target_address);
        let start = ram_target_address as usize;
        let end = start + SIZE_SECTOR as usize;

        let ram_slice= &mut self.memory[start..end];
        self.disk_drive.read_exact(ram_slice).unwrap();
    }

    fn write_raw_data(&mut self, sector: u64, data: &[u8]){
        let disk_offset = sector * SIZE_SECTOR;

        self.disk_drive.seek(SeekFrom::Start(disk_offset)).expect("Seek failed Write Raw Data");
        self.disk_drive.write_all(data).expect("Write failed Write Raw Data");
    }

    pub fn write_to_disk(&mut self, ram_start: usize, ram_end: usize, sector: u64){
        let data = &self.memory[ram_start..ram_end].to_vec();

        self.write_raw_data(sector, data);

        println!("[HARDWARE] Memory range {}..{} saved to sector {}", ram_start, ram_end, sector);
    }

    pub fn write_to_disk_large(&mut self, data: &[u8], start_sector: u64) {
        let mut current_sector = start_sector;
        for chunk in data.chunks(512) {
            self.write_raw_data(current_sector, chunk);
            current_sector += 1;
        }
        println!("[HARDWARE] Large write complete starting at sector {}", start_sector);
    }

    pub fn launch_bios(&mut self, boot_sector:u64) -> bool {
        println!("[BIOS] Launching BIOS...");

        self.load_sector_from_disk(boot_sector, 0x7C00);

        let sig_low = self.memory[0x7C00 + 510];
        let sig_high = self.memory[0x7C00 + 511];

        if sig_low == 0x55 && sig_high == 0xAA {
            println!("[BIOS] BIOS is running");
            self.pc = 0x7C00;
            return true
        }

        println!("[BIOS] No bootable device is found");
        false
    }
    pub fn run(&mut self) {
        self.running = true;
        println!("[CPU] Starting execution loop at PC: 0x{:04X}...", self.pc);

        while self.running {
            if INSTR_START > self.pc || self.pc >= INSTR_END {
                println!("[CPU] Segfault. PC (0x{:04X}) attempted to execute non-code memory.", self.pc);
                self.running = false;
                break;
            }

            let instr = self.consume_u64();
            let opcode = (instr & 0xFFFF) as u16;
            let val1 = ((instr >> 16) & 0xFFFF) as usize;
            let val2 = ((instr >> 32) & 0xFFFF) as u16;
            let val3 = ((instr >> 48) & 0xFFFF) as u16;

            match Opcode::try_from(opcode) {
                Ok(Opcode::Halt) => { println!("[CPU] halt"); self.running = false; break;}
                Ok(Opcode::Add) => {
                    // Run add logic...
                }
                Err(_) => {
                    println!("[CPU ERROR] Unknown opcode '{}'", opcode); self.running = false; break; }
                _ => {println!("[CPU ERROR] Unknown opcode '{}'", opcode); self.running = false; break;}
            }
        }
    }
}