pub const PALETTE: [u32; 16] = [
    0x000000,
    0x555555,
    0xaaaaaa,
    0xffffff,
    0xffaaaa,
    0xaa5555,
    0x550000,
    0x555500,
    0xaaaa55,
    0xaaffaa,
    0x55aa55,
    0x005500,
    0x55aaaa,
    0xaaaaff,
    0x5555aa,
    0xaa55aa,
];

pub fn lookup_palette(color_index: usize) -> u32 {
    let safe_index = color_index & 0x0F;
    PALETTE[safe_index]
}