const VRAM_SIZE: usize = 0x2000; // 8KB
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;

const OAM_SIZE: usize = 0xA0; // 160B
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;

pub struct Gpu {
    bytes: Vec<u8>,
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            bytes: vec![0; VRAM_SIZE + OAM_SIZE],
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }
}
