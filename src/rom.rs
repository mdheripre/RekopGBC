use crate::Result;
use log::info;
use std::fs::File;
use std::io::Read;

const ROM_BANK_SIZE: usize = 0x1000; // 4KB
pub const ROM_START: u16 = 0x0000;
pub const ROM_BANK_END: u16 = 0x7FFF; // 32KB

const ERAM_SIZE: usize = 0x2000; // 8KB
pub const ERAM_START: u16 = 0xA000;
pub const ERAM_END: u16 = 0xBFFF; // 8KB

pub struct Rom {
    bytes: Vec<u8>,
}

pub fn load(path: &str) -> Result<Rom> {
    info!("Opening rom ...");
    let mut buffer = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut buffer)?;

    Ok(Rom { bytes: buffer })
}

impl Rom {
    pub fn read(&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }
}
