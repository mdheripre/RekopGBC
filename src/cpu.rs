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

    fn fetch_byte(&mut self, memory: &MMU) -> u8 {
        let byte = memory.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        byte
    }

    fn handle_interrupts(&mut self, memory: &MMU) {

    }

    pub fn step(&mut self, memory: &mut MMU) -> usize {
        self.handle_interrupts(memory);
        let opcode = self.fetch_byte(memory);

        match opcode {
            0x78 => {
                self.regs.a = self.regs.b;
                4
            }
            0x79 => {
                self.regs.a = self.regs.c;
                4
            }
            _ => 0xFF,
        }
    }
}
