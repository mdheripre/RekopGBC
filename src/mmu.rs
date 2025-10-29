use crate::hram::{Hram, HRAM_END, HRAM_START};
use crate::joypad::Joypad;
use crate::ppu::{Ppu, OAM_END, OAM_START, VRAM_END, VRAM_START};
use crate::rom::{Rom, ERAM_END, ERAM_START, ROM_BANK_END, ROM_START};
use crate::timer::Timer;
use crate::wram::{Wram, ECHO_END, ECHO_START, WRAM_END, WRAM_START};

pub struct Mmu {
    pub rom: Rom,
    pub ppu: Ppu,
    pub wram: Wram,
    pub hram: Hram,
    pub joypad: Joypad,
    pub timer: Timer,
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
            joypad: Joypad::new(),
            timer: Timer::new(),
            inte: 0,
            intf: 0,
            wram_bank: 1,
            vram_bank: 1,
        }
    }

    pub fn do_cycle(&mut self, ticks: u32) -> u32 {
        // let ppu_ticks = ticks / vram_ticks;
        // let cpu_ticks = ticks + vram_ticks;

        self.timer.do_cycle(ticks);
        self.intf |= self.timer.interrupt;
        self.timer.interrupt = 0;

        self.intf |= self.joypad.interrupt;
        self.joypad.interrupt = 0;

        self.ppu.do_cycle(ticks);
        self.intf |= self.ppu.interrupt;
        self.ppu.interrupt = 0;

        return ticks;
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
            0xFF00 => self.joypad.rb(),
            _ => 0xFF,
        }
    }

    pub fn wb(&mut self, a: u16, v: u8) {
        match a {
            ROM_START..=ROM_BANK_END => self.rom.wb(a, v),
            VRAM_START..=VRAM_END => self.ppu.wb(a, v),
            ERAM_START..=ERAM_END => self.rom.wb(a, v),
            WRAM_START..=WRAM_END => self.wram.wb(a, v),
            ECHO_START..=ECHO_END => self.wram.wb(a, v),
            OAM_START..=OAM_END => self.ppu.wb(a, v),
            HRAM_START..=HRAM_END => self.hram.wb(a, v),
            0xFF00 => self.joypad.wb(v),
            // 0xFF46 => {
            //    DMA transfert
            // },
            _ => (),
        };
    }

    pub fn rw(&mut self, address: u16) -> u16 {
        (self.rb(address) as u16) | ((self.rb(address + 1) as u16) << 8)
    }

    pub fn ww(&mut self, a: u16, v: u16) {
        self.wb(a, (v & 0xFF) as u8);
        self.wb(a + 1, (v >> 8) as u8)
    }
}
