use crate::mmu::MMU;
use crate::registers::Registers;
use crate::rom::Rom;
use crate::Result;

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

    fn next_byte(&mut self, memory: &MMU) -> Result<u8> {
        let byte = memory.read(self.regs.pc)?;
        self.regs.pc = self.regs.pc.wrapping_add(1);
        Ok(byte)
    }
}
