pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE; // 127B

pub struct Hram {}

impl Hram {
    pub fn new() -> Hram {
        Hram {}
    }

    pub fn read(&self, address: u16) -> u8 {}
}
