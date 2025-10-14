use crate::{
    cpu::CPU,
    rom::{self},
    Result,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
}
