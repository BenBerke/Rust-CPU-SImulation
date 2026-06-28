use std::fs::File;
use std::future::pending;
use std::io::{Read, Seek, SeekFrom, Write};
use crate::opcodes::*;
use crate::constants::*;

pub struct Core{
    pub regs: [u64; REG_COUNT as usize], // reg0 = return & sys call reg
    pub mem: [u8;SIZE_MEMORY as usize], // Main memory
    pub disk_drive: File,

    pub pc: usize, // Program Counter (points to current instruction)
    pub running: bool,
    pub halted: bool,

    iro: u32
}

impl Core{
    pub fn new(disk_file: File) -> Box<Core>
    { Box::new(Core { regs: [0;REG_COUNT as usize], mem:[0;SIZE_MEMORY as usize], disk_drive: disk_file, pc: 0, running: false, halted: false, iro: 0 }) }

    pub fn consume_byte(&mut self) -> u8 {
        let byte = self.mem[self.pc];
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
        bytes.copy_from_slice(&self.mem[self.pc..self.pc+8]);
        self.pc += 8;
        u64::from_le_bytes(bytes)
    }

    pub fn load_sector_from_disk(&mut self, sector_number: u64, ram_target_address: usize) -> bool{
        let disk_offset = sector_number * SIZE_SECTOR;

        if let Err(err) = self.disk_drive.seek(SeekFrom::Start(disk_offset)) {
            println!("[HARDWARE] Seek failed: {}", err);
            self.running = false;
            return false;
        }

        println!("[HARDWARE] Disk Controller: Streaming {} bytes into Ram Adresss: {}", SIZE_SECTOR, ram_target_address);

        let start = ram_target_address;
        let end = start + SIZE_SECTOR as usize;

        if let Err(err) = self.disk_drive.read_exact(&mut self.mem[start..end]) {
            println!(
                "[DISK ERROR] Failed to read sector {}. Disk image is probably too small or not padded to 512-byte sectors. Error: {}",
                sector_number,
                err
            );

            self.running = false;
            return false;
        }

        true
    }

    pub fn write_raw_data(&mut self, sector: u64, data: &[u8]){
        let disk_offset = sector * SIZE_SECTOR;

        self.disk_drive.seek(SeekFrom::Start(disk_offset)).expect("Seek failed Write Raw Data");
        self.disk_drive.write_all(data).expect("Write failed Write Raw Data");
    }

    pub fn write_to_disk(&mut self, ram_start: usize, ram_end: usize, sector: u64){
        let data = &self.mem[ram_start..ram_end].to_vec();

        self.write_raw_data(sector, data);

        println!("[HARDWARE] Memory range {}..{} saved to sector {}", ram_start, ram_end, sector);
    }

    pub fn write_to_disk_large(&mut self, data: &[u8], start_sector: u64) {
        let disk_offset = start_sector * SIZE_SECTOR;

        let mut buffer =[0u8;SIZE_SECTOR as usize];
        let len = data.len().min(SIZE_SECTOR as usize);
        buffer[..len].copy_from_slice(&data[..len]);

        self.disk_drive.seek(SeekFrom::Start(disk_offset)).expect("Seek failed Write Large");
        self.disk_drive.write_all(&buffer).expect("Write Large failed.");
    }

    pub fn launch_bios(&mut self, boot_sector:u64) -> bool {
        println!("[BIOS] Launching BIOS...");

        if !self.load_sector_from_disk(boot_sector, 0){
            println!("[BIOS] Failed to load boot sector");
            return false;
        }

        // Magic addresses to ensure bios is there
        let sig_low = self.mem[510];
        let sig_high = self.mem[511];

        if sig_low == 0x55 && sig_high == 0xAA {
            println!("[BIOS] BIOS is running");
            self.pc = 0x0;
            return true
        }

        println!("[BIOS] No bootable device is found");
        false
    }

    pub fn write_byte(&mut self, addr: usize, value: u8){
        if addr >= SIZE_MEMORY as usize {
            println!("[CPU] Segfault. Attempted out-of-bounds write at 0x{:05X}", addr);
            self.running = false;
            return;
        }

        self.mem[addr] = value;
    }
    pub fn step(&mut self) {
        use Opcode::*;

        if !self.running || self.halted { return; }

        if self.iro != 0 {

        }

        if INSTR_START > self.pc || self.pc >= INSTR_END {
            println!("[CPU] Segfault. PC (0x{:04X}) attempted to execute non-code memory.", self.pc);
            self.running = false;
            return;
        }

        let instr = self.consume_u64();
        let opcode = (instr & 0xFFFF) as u16;
        let val1 = ((instr >> 16) & 0xFFFF) as usize;
        let val2 = ((instr >> 32) & 0xFFFF) as usize;
        let val3 = ((instr >> 48) & 0xFFFF) as usize;

        match Opcode::try_from(opcode) {
            Ok(Halt) => {
                let exit_code = self.regs[0];
                println!("\n--- [CPU] Program Terminated ---");

                if exit_code == 0{ println!("[STATUS] SUCCESS"); }
                else { println!("[STATUS] Error! Program exited with code: 0x{:04X} ({})", exit_code, exit_code); }

                self.running = false;
                self.halted = true;
            }

            Ok(Add) => { self.regs[val1] = self.regs[val2].wrapping_add(self.regs[val3]); }
            Ok(Sub) => { self.regs[val1] = self.regs[val2].wrapping_sub(self.regs[val3]); }
            Ok(Mul) => { self.regs[val1] = self.regs[val2].wrapping_mul(self.regs[val3]); }

            Ok(Div) => {
                if self.regs[val3] == 0 {
                    println!("[CPU ERROR] Division by zero");
                    self.running = false;
                    return;
                }
                self.regs[val1] = self.regs[val2] / self.regs[val3];
            }

            Ok(SaveDisk) => { self.write_to_disk(val2, val3, val1 as u64)}

            Ok(LD8) => {
                let dest_reg = val1;
                let src_reg = val2;

                if dest_reg >= self.regs.len() || src_reg >= self.regs.len() {
                    println!("[CPU ERROR] Invalid register in LDB.");
                    self.running = false;
                    return;
                }

                let addr = self.regs[src_reg] as usize;

                if addr >= SIZE_MEMORY {
                    println!("[CPU ERROR] LDB out of bounds: 0x{:X}", addr);
                    self.running = false;
                    return;
                }

                self.regs[dest_reg] = self.mem[addr] as u64;
            }

            Ok(LD16) => {
                let dest_reg = val1;
                let src_reg = val2;

                if dest_reg >= self.regs.len() || src_reg >= self.regs.len() {
                    println!("[CPU ERROR] Invalid register in LDW.");
                    self.running = false;
                    return;
                }

                let addr = self.regs[src_reg] as usize;

                if addr + 2 > SIZE_MEMORY {
                    println!("[CPU ERROR] LDW out of bounds: 0x{:X}", addr);
                    self.running = false;
                    return;
                }

                let low = self.mem[addr] as u64;
                let high = self.mem[addr + 1] as u64;

                self.regs[dest_reg] = low | (high << 8);
            }

            Ok(LD64) => {
                let dest_reg = val1;
                let src_reg = val2;

                if dest_reg >= self.regs.len() || src_reg >= self.regs.len() {
                    println!("[CPU ERROR] Invalid register in LDQ.");
                    self.running = false;
                    return;
                }

                let addr = self.regs[src_reg] as usize;

                if addr + 8 > SIZE_MEMORY {
                    println!("[CPU ERROR] LDQ out of bounds: 0x{:X}", addr);
                    self.running = false;
                    return;
                }

                let bytes = &self.mem[addr..addr + 8];
                self.regs[dest_reg] = u64::from_le_bytes(bytes.try_into().unwrap());
            }

            Ok(DTM) => { // Disk to Memory
                let mut ram_dest = val1;
                let start_sector = val2 as u64;
                let sector_count = self.regs[val3];

                for i in 0..sector_count {
                    let current_sector = start_sector + i;

                    if ram_dest + (SIZE_SECTOR as usize) <= SIZE_MEMORY {
                        self.load_sector_from_disk(current_sector, ram_dest);
                        ram_dest += SIZE_SECTOR as usize;
                    }
                    else {
                        println!("[CPU ERROR] DTM DMA target burst went out of RAM limits.");
                        self.running = false;
                        break;
                    }
                }
            }

            Ok(LoadImm) => {
                let dest_reg = val1;

                if dest_reg >= self.regs.len() {
                    println!("[CPU ERROR] Invalid register in LDI.");
                    self.running = false;
                    return;
                }

                let imm32 = ((val2 as u64) << 16) | (val3 as u64);
                self.regs[dest_reg] = imm32;
            }

            Ok(ST8) => {
                let addr_reg = val1;
                let src_reg = val2;

                if addr_reg >= self.regs.len() || src_reg >= self.regs.len() {
                    println!("[CPU ERROR] Invalid register in STB.");
                    self.running = false;
                    return;
                }

                let addr = self.regs[addr_reg] as usize;
                let value = self.regs[src_reg];

                self.write_byte(addr, (value & 0xFF) as u8);
            }
            Ok(ST16) => {
                let addr_reg = val1;
                let src_reg = val2;

                if addr_reg >= self.regs.len() || src_reg >= self.regs.len() {
                    println!("[CPU ERROR] Invalid register in STB.");
                    self.running = false;
                    return;
                }

                let addr = self.regs[addr_reg] as usize;
                let value = self.regs[src_reg];

                self.write_byte(addr, (value & 0xFF) as u8);
                self.write_byte(addr + 1, ((value >> 8) & 0xFF) as u8);
            }
            Ok(ST64) => {
                let addr_reg = val1;
                let src_reg = val2;

                if addr_reg >= self.regs.len() || src_reg >= self.regs.len() {
                    println!("[CPU ERROR] Invalid register in STB.");
                    self.running = false;
                    return;
                }

                let addr = self.regs[addr_reg] as usize;
                let value = self.regs[src_reg];

                for i in 0..8{ self.write_byte(addr + i, ((value >> (i * 8)) & 0xFF) as u8); }
            }

            Ok(Jmp) => {self.pc = val1 }
            Ok(JmpAbs) => { self.pc = val1 }
            Ok(JumpZero) => { if self.regs[val2] == 0 { self.pc = val1} }

            Err(_) => {
                println!("[CPU ERROR] Unknown opcode '{}'", opcode); self.running = false; return; }
            _ => {println!("[CPU ERROR] Unknown opcode '{}'", opcode); self.running = false; return;}
        }
    }
}