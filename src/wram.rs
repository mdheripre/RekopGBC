const WRAM_BANK_SIZE: usize = 0x1000; // 4KB
const WRAM_BANK_COUNT: usize = 8;
const WRAM_SIZE: usize = WRAM_BANK_SIZE * WRAM_BANK_COUNT; // 32KB

pub const WRAM_START: u16 = 0xC000;
pub const WRAM_END: u16 = 0xDFFF; // 8KB

pub const ECHO_START: u16 = 0xE000;
pub const ECHO_END: u16 = 0xFDFF; // 7.5KB

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Wram {
    #[serde(with = "BigArray")]
    banks: [[u8; WRAM_BANK_SIZE]; WRAM_BANK_COUNT],
    current_bank: usize,
}

impl Wram {
    pub fn new() -> Wram {
        Wram {
            banks: [[0; WRAM_BANK_SIZE]; WRAM_BANK_COUNT],
            current_bank: 1,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xC00..=0xCFFF => {
                let offset = (address - 0xC00) as usize;
                self.banks[0][offset]
            }
            0xD00..=0xDFFF => {
                let offset = (address - 0xD00) as usize;
                self.banks[self.current_bank][offset]
            }
            _ => panic!("Invalid WRAM address"),
        }
    }
}
