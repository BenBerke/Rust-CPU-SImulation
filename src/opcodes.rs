macro_rules! define_opcodes {
    ($( $name:ident = $val:expr => $str_name:expr ),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u16)]
        pub enum Opcode {
            $( $name = $val, )*
        }

        impl Opcode {
            // This function takes "HALT", "LOAD", etc. and converts it to u64
            pub fn from_str(s: &str) -> Option<u64> {
                match s {
                    $( $str_name => Some(Opcode::$name as u64), )*
                    _ => None,
                }
            }
        }

        impl TryFrom<u16> for Opcode {
            type Error = ();
            fn try_from(v: u16) -> Result<Self, Self::Error> {
                match v {
                    $( $val => Ok(Opcode::$name), )*
                    _ => Err(()),
                }
            }
        }
    };
}

define_opcodes! {
    Halt     = 0 => "HLT",      // Halts execution
    Add      = 2 => "ADD",      // reg1 reg2 reg3 / reg1 = reg2 + reg3

    Jmp      = 4 => "JMP",      // sym / pc = sym
    SaveDisk = 5 => "SDK",      // reg1 reg2 reg3 / drive[reg1] = memory[reg2..reg3]
    Sub      = 6 => "SUB",      // reg1 reg2 reg3 / reg1 = reg2 + reg3
    Mul      = 7 => "MUL",      // reg1 reg2 reg3 / reg1 = reg2 + reg3
    Div      = 8 => "DIV",      // reg1 reg2 reg3 / reg1 = reg2 + reg3
    JmpAbs   = 9 => "JAB",      // imm32 / pc = imm32
    JumpZero = 10 => "JZF",     // sym reg / reg = 0 -> pc = sym
    LoadImm     = 11 => "LDI",     // reg imm / reg = imm

    DTM     = 13 => "DTM",     // imm32a imm32b reg / mem start, start sector, sector count
    LD8 = 14 => "LDB", // reg reg / Load byte from memory
    LD16  = 15 => "LDW", // reg reg / Load word from memory
    LD64  = 16 => "LDQ", // reg reg / Load qword from memory
    ST8  = 17 => "STB", // reg reg / mem[addr_reg] = low 8 bits of value_reg
    ST16 = 18 => "STW", // reg reg / mem[addr_reg..addr_reg+2] = low 16 bits
    ST64 = 19 => "STQ", // reg reg / mem[addr_reg..addr_reg+8] = full 64 bits
    JGE = 20 => "JGE" // sym reg1 reg2 / jump if reg1 > reg2
}