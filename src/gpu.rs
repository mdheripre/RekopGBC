pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF; // 8KB

pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F; // 160B

pub struct Gpu {}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {}
    }

    pub fn read(&self, address: u16) -> u8 {}
}
