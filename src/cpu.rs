use crate::mmu::MMU;
use crate::registers::Registers;
use crate::rom::Rom;

pub struct CPU {
    regs: Registers,
    pub mmu: MMU,
}

impl CPU {
    pub fn new(rom: Rom) -> CPU {
        CPU {
            regs: Registers::new(),
            mmu: MMU::new(rom),
        }
    }

    fn next_byte(&mut self, memory: &MMU) -> u8 {
        let byte = memory.read(self.regs.pc as u16);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        byte
    }
}
