pub struct Timer {
    pub ticks: u64,
}

impl Timer {
    pub fn new() -> Self { Self{ticks: 0} }

    pub fn tick(&mut self, amount: u64) { self.ticks = self.ticks.wrapping_add(amount); }

    pub fn read_byte(&self, offset: usize) -> u8 {
        let bytes = self.ticks.to_le_bytes();
        bytes[offset]
    }
}