use crate::{
    cpu::CPU,
    device,
    rom::{self, Rom},
};

pub struct Device {
    cpu: CPU,
    save_state: Option<String>,
}

impl Device {
    pub fn new(romname: &str, save_state: Option<String>) -> Device {
        let cart = rom::load(romname);
        Device {
            cpu: CPU::new(cart),
            save_state,
        }
    }
}
