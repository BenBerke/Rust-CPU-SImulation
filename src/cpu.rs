use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use crate::opcodes::*;

const SIZE_SECTOR: u64 = 512;
const SIZE_MEMORY: u64 = 1 * 1024 * 8;
const REG_COUNT: u8 = 8;

pub struct Core{
    registers: [u16; REG_COUNT as usize],
    memory: [u8;SIZE_MEMORY as usize], // Main memory
    disk_drive: File,

    pc: usize, // Program Counter (points to current instruction)
    running: bool
}

impl Core{
    pub fn new(disk_file: File) -> Box<Core>
    { Box::new(Core { registers: [0;REG_COUNT as usize], memory:[0;SIZE_MEMORY as usize], disk_drive: disk_file, pc: 0, running: false }) }

    pub fn consume_byte(&mut self) -> u8 {
        let byte = self.memory[self.pc];
        self.pc += 1;
        byte
    }

    pub fn consume_u16(&mut self) -> u16 {
        let high = self.consume_byte() as u16;
        let low = self.consume_byte() as u16;
        (high << 8) | low
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
    pub fn run(&mut self) {
        self.running = true;

        while self.running {
            let opcode = self.memory[self.pc];
            match opcode {
                OP_HALT => {
                    println!("HALT");
                    self.running = false
                },
                OP_LOAD => {
                    // [LDR] [reg_idx (1)] [val_high (1)] [val_low (1)]
                    let reg_idx = self.consume_byte() as usize;
                    let value = self.consume_u16();

                    self.registers[reg_idx] = value;

                    println!("LOADED {} TO {}", value, reg_idx);
                }
                OP_ADD => {
                    // [OP_ADD] [reg_dest (1)] [reg_src1 (1)] [reg_src2 (1)]
                    let reg_dest = self.consume_byte() as usize;
                    let reg_src1 = self.consume_byte() as usize;
                    let reg_src2 = self.consume_byte() as usize;

                    self.registers[reg_dest] = self.registers[reg_src1] + self.registers[reg_src2];

                    println!("ADD: R{} = R{} + R{}", reg_dest, reg_src1, reg_src2);

                    self.pc += 4;
                },
                OP_STORE=> {
                    //  [OP_STORE] [reg_src (1)] [addr_high (1)] [addr_low (1)]
                    let reg_src = self.consume_byte() as usize;
                    let ram_address = self.consume_u16() as usize;

                    let reg_val = self.registers[reg_src];

                    self.memory[ram_address] = (reg_val >> 8) as u8;
                    self.memory[ram_address + 1] = (reg_val & 0xFF) as u8;

                    println!("STORE: Wrote R{} ({}) into RAM address {}", reg_src, reg_src, ram_address);
                },
                _=>{
                    println!("Unknown opcode {}", opcode);
                    self.running = false
                }
            }
        }
    }
}