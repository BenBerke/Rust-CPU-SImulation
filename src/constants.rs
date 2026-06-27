pub const SIZE_SECTOR: u64 = 512;
pub const SIZE_MEMORY: usize = 512 * 1024; // 524,288 Bytes (512KB)
pub const REG_COUNT: u8 = 8;

// Display Config (320x240 Indexed Palette)
pub const SCREEN_WIDTH: usize = 320;
pub const SCREEN_HEIGHT: usize = 240;
pub const SCREEN_VIRTUAL_WIDTH: usize = SCREEN_WIDTH * 3;
pub const SCREEN_VIRTUAL_HEIGHT: usize = SCREEN_HEIGHT * 3;

// =========================================================================
// MEMORY LAYOUT SEGMENTS (HEX ENCODED)
// =========================================================================

// Bank 0: Instructions / Executable Code (128 KB)
pub const INSTR_START: usize = 0x00000;
pub const INSTR_END: usize   = 0x1FFFF;

// Bank 1: Kernel Global Data & Data Scratchpad (128 KB)
pub const DATA_START: usize = 0x20000;
pub const DATA_END: usize = 0x3FFFF;

// Bank 2: Memory-Mapped I/O & Graphics (128 KB)
pub const MMIO_START: usize = 0x40000;
pub const VRAM_START: usize = 0x40000; // VRAM sits at the base of MMIO

pub const VRAM_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT; // 76,800 Bytes (~75KB)
pub const VRAM_END: usize = VRAM_START + VRAM_SIZE; // Ends at 0x52C00
pub const MMIO_END: usize = 0x5FFFF; // Remaining ~51KB for audio/input/palettes
pub const IO_INPUT_START: usize = VRAM_END; // 0x52C00
pub const IO_INPUT_SIZE: usize = 128;

// Bank 3: User Space Heap / Graphics Backbuffer (112 KB)
pub const USER_START: usize = 0x60000;
pub const USER_END: usize = 0x7BFFF;

// Hardware Stack Region (16 KB)
// The stack pointer (SP) will start at 0x7FFFF and grow DOWNWARDS
pub const STACK_START: usize = 0x7C000;
pub const STACK_END: usize = 0x7FFFF;

pub const SECTION_DATA: &str = "data";
pub const SECTION_TEXT: &str = "text";

pub const BOOTLOADER_BASE_ADDRESS: usize = 0;
pub const KERNEL_LOAD_ADDRESS: usize = SIZE_SECTOR as usize; // 512
pub const KERNEL_HEADER_SIZE: usize = 4;
pub const KERNEL_CODE_ADDRESS: usize = KERNEL_LOAD_ADDRESS + KERNEL_HEADER_SIZE; // 516