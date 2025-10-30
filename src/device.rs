use crate::{
    cpu::CPU,
    rom::{self},
    Result,
};

pub struct Device {
    cpu: CPU,
    save_state: Option<String>,
}

impl Device {
    pub fn new(romname: &str, save_state: Option<String>) -> Result<Device> {
        let cart = rom::load(romname)?;
        Ok(Device {
            cpu: CPU::new(cart),
            save_state,
        })
    }

    pub fn do_cycle(&mut self) -> u32 {
        self.cpu.do_cycle()
    }

    pub fn ppu_data(&self) -> Vec<u32> {
        self.cpu.mmu.ppu.get_framebuffer()
    }
}
