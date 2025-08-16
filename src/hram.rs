pub const HRAM_SIZE: usize = 0x7F; // 127B
pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;

pub struct Hram {
    bytes: Vec<u8>
}

impl Hram {
    pub fn new() -> Hram {
        Hram {
            bytes: vec![0; HRAM_SIZE],
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }
}
