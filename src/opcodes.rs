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
    Halt     = 0 => "HALT",
    Load     = 1 => "LOAD",
    Add      = 2 => "ADD",
    Store    = 3 => "STORE",
    Jmp      = 4 => "JMP",
    SaveDisk = 5 => "SAVEDISK",
}