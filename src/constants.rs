
pub const SIZE_SECTOR: u64 = 512;
pub const SIZE_MEMORY: u64 = 48 * 1024;
pub const REG_COUNT: u8 = 8;
// Bank 0: Instructions only
pub const INSTR_START: usize = 0x0000;
pub const INSTR_END: usize = 0x4000; // 16KB boundary

// Bank 1: Normal Data Memory (RAM scratchpad & Stack)
pub const DATA_START: usize = 0x4000;
pub const DATA_END: usize = 0x8000;

// Bank 2: Memory-Mapped I/O base address
pub const MMIO_START: usize = 0x8000;
pub const MMIO_END: usize = 0xC000;    // 48KB boundary