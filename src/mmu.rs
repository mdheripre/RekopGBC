use crate::hram::{Hram, HRAM_END, HRAM_START};
use crate::ppu::{Ppu, OAM_END, OAM_START, VRAM_END, VRAM_START};
use crate::rom::{Rom, ERAM_END, ERAM_START, ROM_BANK_END, ROM_START};
use crate::wram::{Wram, ECHO_END, ECHO_START, WRAM_END, WRAM_START};

pub struct Mmu {
    rom: Rom,
    ppu: Ppu,
    wram: Wram,
    hram: Hram,
    pub inte: u8,
    pub intf: u8,
    wram_bank: u8,
    vram_bank: u8,
}

impl Mmu {
    pub fn new(_rom: Rom) -> Mmu {
        Mmu {
            rom: _rom,
            ppu: Ppu::new(),
            wram: Wram::new(),
            hram: Hram::new(),
            inte: 0,
            intf: 0,
            wram_bank: 1,
            vram_bank: 1,
        }
    }

    pub fn rb(&mut self, a: u16) -> u8 {
        match a {
            ROM_START..=ROM_BANK_END => self.rom.rb(a),
            VRAM_START..=VRAM_END => self.ppu.rb(a),
            ERAM_START..=ERAM_END => self.rom.rb(a),
            WRAM_START..=WRAM_END => self.wram.rb(a),
            ECHO_START..=ECHO_END => self.wram.rb(a),
            OAM_START..=OAM_END => self.ppu.rb(a),
            HRAM_START..=HRAM_END => self.hram.rb(a),
            _ => 0xFF,
        }
    }

    pub fn wb(&mut self, a: u16, b: u8) {
        match a {
            ROM_START..=ROM_BANK_END => self.rom.rb(a),
            VRAM_START..=VRAM_END => self.ppu.rb(a),
            ERAM_START..=ERAM_END => self.rom.rb(a),
            WRAM_START..=WRAM_END => self.wram.rb(a),
            ECHO_START..=ECHO_END => self.wram.rb(a),
            OAM_START..=OAM_END => self.ppu.rb(a),
            HRAM_START..=HRAM_END => self.hram.rb(a),
            // 0xFF46 => {
            //    DMA transfert
            // },
            _ => 0xFF,
        };
    }
}
