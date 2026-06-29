use crate::constants::*;
use crate::hardware::bus::Bus;
use crate::opcodes::*;
pub struct Core{
    pub regs: [u64; REG_COUNT as usize], // reg0 = return / sys call reg

    pub pc: usize, // Program Counter (points to current instruction)
    pub running: bool,
    pub halted: bool,

    iro: u32
}

impl Core{
    pub fn new() -> Self { Self { regs: [0; REG_COUNT as usize], pc: 0, running: false, halted: false, iro: 0, } }

    pub fn step(&mut self, bus: &mut Bus) {
        use Opcode::*;

        if !self.running || self.halted { return; }

        if self.iro != 0 {
            // todo Interrupt handling
        }

        if INSTR_START > self.pc || self.pc >= INSTR_END {
            println!("[CPU] Segfault. PC (0x{:04X}) attempted to execute non-code memory.", self.pc);
            self.running = false;
            return;
        }

        if self.pc + 8 > SIZE_MEMORY {
            println!("[CPU] Segfault. Tried to fetch instruction outside memory at 0x{:05X}", self.pc);
            self.running = false;
            return;
        }

        let mut instr_bytes = [0u8; 8];
        instr_bytes.copy_from_slice(&bus.mem[self.pc..self.pc + 8]);
        self.pc += 8;

        let instr = u64::from_le_bytes(instr_bytes);

        let opcode = (instr & 0xFFFF) as u16;
        let val1 = ((instr >> 16) & 0xFFFF) as usize;
        let val2 = ((instr >> 32) & 0xFFFF) as usize;
        let val3 = ((instr >> 48) & 0xFFFF) as usize;

        match Opcode::try_from(opcode) {
            Ok(Halt) => {
                let exit_code = self.regs[0];

                println!("\n--- [CPU] Program Terminated ---");

                if exit_code == 0 {
                    println!("[STATUS] SUCCESS");
                } else {
                    println!(
                        "[STATUS] Error! Program exited with code: 0x{:04X} ({})",
                        exit_code,
                        exit_code
                    );
                }

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

                self.regs[dest_reg] = bus.read_byte(addr) as u64;
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

                if addr + 2 > SIZE_MEMORY as usize {
                    println!("[CPU ERROR] LDW out of bounds: 0x{:X}", addr);
                    self.running = false;
                    return;
                }

                let low = bus.read_byte(addr) as u64;
                let high = bus.read_byte(addr + 1) as u64;

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

                self.regs[dest_reg] = bus.read_u64(addr);
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

                bus.mem[addr] = (value & 0xFF) as u8;
            }

            Ok(ST16) => {
                let addr_reg = val1;
                let src_reg = val2;

                if addr_reg >= self.regs.len() || src_reg >= self.regs.len() {
                    println!("[CPU ERROR] Invalid register in STW.");
                    self.running = false;
                    return;
                }

                let addr = self.regs[addr_reg] as usize;
                let value = self.regs[src_reg];

                if addr + 2 > SIZE_MEMORY {
                    println!("[CPU ERROR] STW out of bounds: 0x{:X}", addr);
                    self.running = false;
                    return;
                }

                bus.mem[addr] = (value & 0xFF) as u8;
                bus.mem[addr + 1] = ((value >> 8) & 0xFF) as u8;
            }

            Ok(ST64) => {
                let addr_reg = val1;
                let src_reg = val2;

                if addr_reg >= self.regs.len() || src_reg >= self.regs.len() {
                    println!("[CPU ERROR] Invalid register in STQ.");
                    self.running = false;
                    return;
                }

                let addr = self.regs[addr_reg] as usize;
                let value = self.regs[src_reg];

                if addr + 8 > SIZE_MEMORY {
                    println!("[CPU ERROR] STQ out of bounds: 0x{:X}", addr);
                    self.running = false;
                    return;
                }

                for i in 0..8 { bus.mem[addr + i] = ((value >> (i * 8)) & 0xFF) as u8; }
            }

            Ok(Jmp) => { self.pc = val1; } // Jumps using a register
            Ok(JmpAbs) => { self.pc = val1; } // Jumps with a literal

            Ok(JumpZero) => {
                if val2 >= self.regs.len() {
                    println!("[CPU ERROR] Invalid register in JZF.");
                    self.running = false;
                    return;
                }

                if self.regs[val2] == 0 {
                    self.pc = val1;
                }
            }

            Ok(DTM) => {
                let mut ram_dest = val1;
                let start_sector = val2 as u64;

                if val3 >= self.regs.len() {
                    println!("[CPU ERROR] Invalid register in DTM.");
                    self.running = false;
                    return;
                }

                let sector_count = self.regs[val3];

                for i in 0..sector_count {
                    let current_sector = start_sector + i;

                    if ram_dest + SIZE_SECTOR as usize > SIZE_MEMORY as usize {
                        println!("[CPU ERROR] DTM target went out of RAM limits.");
                        self.running = false;
                        return;
                    }

                    if !bus.load_sector_from_disk(current_sector, ram_dest) {
                        self.running = false;
                        return;
                    }

                    ram_dest += SIZE_SECTOR as usize;
                }
            }

            Ok(SaveDisk) => {
                if !bus.write_to_disk(val2, val3, val1 as u64) {
                    self.running = false;
                    return;
                }
            }

            Ok(JGE) => { if self.regs[val2] >= self.regs[val3] { self.pc = val1; } }

            Err(_) => {
                println!("[CPU ERROR] Unknown opcode '{}'", opcode);
                self.running = false;
            }
        }
    }
}