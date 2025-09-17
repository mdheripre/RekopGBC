use crate::mmu::MMU;
use crate::registers::Registers;
use crate::rom::Rom;
use crate::registers::CpuFlag::{C, H, N, Z};

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

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.mmu.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        byte
    }

    fn handle_interrupts(&mut self) {}

    pub fn step(&mut self) -> usize {
        self.handle_interrupts();
        let opcode = self.fetch_byte();

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

    fn call(&mut self) -> u32 {
        let opcode = self.fetch_byte();

        match opcode {
            0xFF => {
                let v = self.fetch_byte();
                self.alu_cp(v);
                2
            }
            other => 0xFF,
        }
    }

    fn alu_add(&mut self, b: u8, usec: bool) {
        let c = if usec && self.regs.get_flag(C) { 1 } else { 0 };
        let a = self.regs.a;
        let r = a.wrapping_add(b).wrapping_add(c);

        self.regs.flag(Z, r == 0);
        self.regs.flag(H, (a & 0xF) + (b & 0xF) + c > 0xF);
        self.regs.flag(N, false);
        self.regs
            .flag(C, (a as u16) + (b as u16) + (c as u16) > 0xFF);
        self.regs.a = r;
    }

    fn alu_sub(&mut self, b: u8, usec: bool) {
        let c = if usec && self.regs.get_flag(C) { 1 } else { 0 };
        let a = self.regs.a;
        let r = a.wrapping_sub(b).wrapping_sub(c);
        self.regs.flag(Z, r == 0);
        self.regs.flag(H, (a & 0x0F) < (b & 0x0F) + c);
        self.regs.flag(N, true);
        self.regs.flag(C, (a as u16) < (b as u16) + (c as u16));
        self.regs.a = r;
    }

    fn alu_and(&mut self, b: u8) {
        let r = self.regs.a & b;
        self.regs.flag(Z, r == 0);
        self.regs.flag(C, false);
        self.regs.flag(H, false);
        self.regs.flag(N, false);
        self.regs.a = r;
    }

    fn alu_or(&mut self, b: u8) {
        let r = self.regs.a | b;
        self.regs.flag(Z, r == 0);
        self.regs.flag(C, false);
        self.regs.flag(H, false);
        self.regs.flag(N, false);
        self.regs.a = r;
    }

    fn alu_xor(&mut self, b: u8) {
        let r = self.regs.a ^ b;
        self.regs.flag(Z, r == 0);
        self.regs.flag(C, false);
        self.regs.flag(H, false);
        self.regs.flag(N, false);
        self.regs.a = r;
    }

    fn alu_cp(&mut self, b: u8) {
        let r = self.regs.a;
        self.alu_sub(b, false);
        self.regs.a = r;
    }

    fn alu_inc(&mut self, a: u8) -> u8 {
        let r = a.wrapping_add(1);
        self.regs.flag(Z, r == 0);
        self.regs.flag(H, (a & 0x0F) + 1 > 0x0F);
        self.regs.flag(N, false);
        r
    }

    fn alu_dec(&mut self, a: u8) -> u8 {
        let r = a.wrapping_sub(1);
        self.regs.flag(Z, r == 0);
        self.regs.flag(H, (a & 0x0F) == 0x0F);
        self.regs.flag(N, true);
        r
    }

    fn alu_add16(&mut self, b: u16) {
        let a = self.regs.hl();
        let r = a.wrapping_add(b);
        self.regs.flag(H, (a & 0x0FFF) + (b & 0x0FFF) > 0x0FFF);
        self.regs.flag(N, false);
        self.regs.flag(C, a > 0xFFFF - b);
        self.regs.sethl(r);
    }

    fn alu_add16imm(&mut self, a:u16) -> u16 {
        let b = self.fetch_byte() as i8 as i16 as u16;
        self.regs.flag(N, false);
        self.regs.flag(Z, false);
        self.regs.flag(H, (a & 0x000F) + (b & 0x000F) > 0x000F);
        self.regs.flag(C, (a & 0x00F) + (b & 0x00F) > 0x00F);
        a.wrapping_add(b)
    }

    fn alu_swap(&mut self, a: u8) -> u8 {
        self.regs.flag(Z, a == 0);
        self.regs.flag(C, false);
        self.regs.flag(H, false);
        self.regs.flag(N, false);
        a.rotate_left(4)
    }

    fn alu_srflag_update(&mut self, r: u8, c: bool) {
        self.regs.flag(Z, r == 0);
        self.regs.flag(H, false);
        self.regs.flag(N, false);
        self.regs.flag(C, c);
    }

    fn alu_rlc(&mut self, a:u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if c { 0x01 } else { 0x00 });
        self.alu_srflag_update(r, c);
        r
    }

    fn alu_rl(&mut self, a:u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if self.regs.get_flag(C) { 0x01 } else { 0x00 });
        self.alu_srflag_update(r, c);
        r
    }

    fn alu_rrc(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = (a >> 1) | (if c { 0x80 } else { 0x00 });
        self.alu_srflag_update(r, c);
        r
    }

    fn alu_rr(&mut self, a:u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a >> 1) | (if self.regs.get_flag(C) { 0x80 } else { 0x00 });
        self.alu_srflag_update(r, c);
        r
    }

    fn alu_sla(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = a << 1;
        self.alu_srflag_update(r, c);
        r
    }

    fn alu_sra(&mut self, a: u8) -> u8 {
        let c = a &0x01 == 0x01;
        let r = (a >> 1) | (a & 0x80);
        self.alu_srflag_update(r, c);
        r
    }

    fn alu_srl(&mut self, a: u8) -> u8 {
        let c = a & 0x01 == 0x01;
        let r = a >> 1;
        self.alu_srflag_update(r, c);
        r
    }

    fn alu_bit(&mut self, a: u8, b: u8) {
        let r = a & (1 << (b as u32)) == 0;
        self.regs.flag(N, false);
        self.regs.flag(H, true);
        self.regs.flag(Z, r);
    }

    fn alu_daa(&mut self) {
        let mut a = self.regs.a;

        let mut adjust = if self.regs.get_flag(C){ 0x60 } else { 0x00 };
        if self.regs.get_flag(H) {
            adjust |= 0x06;
        }
        if !self.regs.get_flag(N) {
            if a & 0x0F > 0x09{
                adjust |= 0x06;
            }
            if a > 0x99 {
                adjust |= 0x60;
            }
            a = a.wrapping_add(adjust);
        } else {
            a = a.wrapping_sub(adjust);
        }

        self.regs.flag(C, adjust >= 0x60);
        self.regs.flag(H, false);
        self.regs.flag(Z, a == 0);
        self.regs.a = a;
    }

    fn cpu_jr(&mut self) {
        let n = self.fetch_byte() as i8;
        self.regs.pc = ((self.regs.pc as u32 as i32) + (n as i32)) as u16;
    }
}
