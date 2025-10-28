pub const HRAM_SIZE: usize = 0x7F; // 127B
pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;

pub struct Hram {
    bytes: [u8; HRAM_SIZE],
}

impl Hram {
    pub fn new() -> Hram {
        Hram {
            bytes: [0; HRAM_SIZE],
        }
    }

    pub fn rb(&self, a: u16) -> u8 {
        self.bytes[a as usize]
    }

    pub fn wb(&mut self, a: u16, v: u8) {
        self.bytes[a as usize] = v
    }
}
