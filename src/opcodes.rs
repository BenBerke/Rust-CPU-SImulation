#[derive(Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Opcode {
    Halt = 0,
    Load = 1,
    Add = 2,
    Store = 3,
    Jmp = 4,
}

impl TryFrom<u16> for Opcode {
    type Error = ();
    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Opcode::Halt),
            1 => Ok(Opcode::Load),
            2 => Ok(Opcode::Add),
            3 => Ok(Opcode::Store),
            4 => Ok(Opcode::Jmp),
            _ => Err(()),
        }
    }
}