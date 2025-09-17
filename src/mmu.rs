use crate::error::MmuError;
use crate::gpu::{Gpu, OAM_END, OAM_START, VRAM_END, VRAM_START};
use crate::hram::{Hram, HRAM_END, HRAM_START};
use crate::rom::{Rom, ERAM_END, ERAM_START, ROM_BANK_END, ROM_START};
use crate::wram::{Wram, ECHO_END, ECHO_START, WRAM_END, WRAM_START};
use crate::{EmulatorError, Result};

pub struct MMU {
    rom: Rom,
    gpu: Gpu,
    wram: Wram,
    hram: Hram,
}

impl MMU {
    pub fn new(_rom: Rom) -> MMU {
        MMU {
            rom: _rom,
            gpu: Gpu::new(),
            wram: Wram::new(),
            hram: Hram::new(),
        }
    }

    pub fn read(&self, address: u16) -> Result<u8> {
        match address {
            ROM_START..=ROM_BANK_END => Ok(self.rom.read(address)),
            VRAM_START..=VRAM_END => Ok(self.gpu.read(address)),
            ERAM_START..=ERAM_END => Ok(self.rom.read(address)),
            WRAM_START..=WRAM_END => Ok(self.wram.read(address)),
            ECHO_START..=ECHO_END => Ok(self.wram.read(address)),
            OAM_START..=OAM_END => Ok(self.gpu.read(address)),
            HRAM_START..=HRAM_END => Ok(self.hram.read(address)),
            _ => Err(MmuError::InvalidMemoryRead(address).into()),
        }
    }
}
