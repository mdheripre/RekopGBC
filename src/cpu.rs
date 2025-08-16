use crate::mmu::MMU;
use crate::registers::Registers;

pub struct CPU {
    regs: Registers,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            regs: Registers::new(),
        }
    }

    fn next_byte(&mut self, memory: &MMU) -> u8 {
        let byte = memory.read(self.regs.pc as u16);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        byte
    }
}
