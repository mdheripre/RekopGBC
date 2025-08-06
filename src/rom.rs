use std::fs::File;
use std::io::Read;

pub const ROM_START: u16 = 0x0000;
pub const ROM_BANK_END: u16 = 0x7FFF; // 32KB

pub const ERAM_START: u16 = 0xA000;
pub const ERAM_END: u16 = 0xBFFF; // 8KB

pub struct Rom {
    bytes: Vec<u8>,
}

pub fn load(path: String) -> Rom {
    let mut buffer = Vec::new();
    let mut file = File::open(path).expect("Invalid ROM path");
    file.read_to_end(&mut buffer).expect("Unable to read ROM");

    Rom { bytes: buffer }
}

impl Rom {
    pub fn read(&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }
}
