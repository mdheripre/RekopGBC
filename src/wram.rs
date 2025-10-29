const WRAM_BANK_SIZE: usize = 0x1000; // 4KB
const WRAM_BANK_COUNT: usize = 8;
const WRAM_SIZE: usize = WRAM_BANK_SIZE * WRAM_BANK_COUNT; // 32KB

pub const WRAM_START: u16 = 0xC000;
pub const WRAM_END: u16 = 0xDFFF; // 8KB

pub const ECHO_START: u16 = 0xE000;
pub const ECHO_END: u16 = 0xFDFF; // 7.5KB

pub struct Wram {
    wram: [[u8; WRAM_BANK_SIZE]; WRAM_BANK_COUNT],
    wram_bank: usize,
}

impl Wram {
    pub fn new() -> Wram {
        Wram {
            wram: [[0; WRAM_BANK_SIZE]; WRAM_BANK_COUNT],
            wram_bank: 1,
        }
    }

    pub fn bank(&self) -> usize {
        self.wram_bank
    }

    pub fn rb(&self, address: u16) -> u8 {
        match address {
            0xC00..=0xCFFF => {
                let offset = (address - 0xC00) as usize;
                self.wram[0][offset]
            }
            0xD00..=0xDFFF => {
                let offset = (address - 0xD00) as usize;
                self.wram[self.wram_bank][offset]
            }
            _ => panic!("Invalid WRAM address"),
        }
    }

    pub fn wb(&mut self, a: u16, v: u8) {
        self.wram[self.wram_bank][a as usize] = v
    }
}
