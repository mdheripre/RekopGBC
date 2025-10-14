use serde::de::value;

use crate::mmu::MMU;
use crate::registers::CpuFlag::{C, H, N, Z};
use crate::registers::Registers;
use crate::rom::Rom;

pub struct CPU {
    regs: Registers,
    pub mmu: MMU,
    halted: bool,
    halt_bug: bool,
    ime: bool,
    setdi: u32,
    setei: u32,
}

impl CPU {
    pub fn new(rom: Rom) -> CPU {
        CPU {
            regs: Registers::new(),
            mmu: MMU::new(rom),
            halted: false,
            halt_bug: false,
            ime: true,
            setdi: 0,
            setei: 0,
        }
    }

    pub fn do_cycle(&mut self) -> u32 {
        let ticks = self.do_cycle() * 4;
        self.mmu.do_cycle(ticks);
    }

    fn docycle(&mut self) -> u32 {
        self.updateime();
        match self.handle_interrupts() {
            0 => {}
            n => return n,
        };

        if self.halted {
            1
        } else {
            self.call()
        }
    }

    fn updateime(&mut self) {
        self.setdi = match self.setdi {
            2 => 1,
            1 => {
                self.ime = false;
                0
            }
            _ => 0,
        };
        self.setei = match self.setei {
            2 => 1,
            1 => {
                self.ime = true;
                0
            }
            _ => 0,
        };
    }

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.mmu.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        byte
    }

    fn fetchword(&mut self) -> u16 {
        let w = self.mmu.rw(self.regs.pc);
        self.regs.pc += 2;
        w
    }

    fn handle_interrupts(&mut self) -> u32 {
        if self.ime == false && self.halted == false {
            return 0;
        }

        let triggered = self.mmu.inte & self.mmu.intf & 0x1F;
        if triggered == 0 {
            return 0;
        }

        self.halted = false;
        if self.ime == false {
            return 0;
        }
        self.ime = false;

        let n = triggered.trailing_zeros();
        if n >= 5 {
            panic!("Invalid interrupt triggered");
        }
        self.mmu.intf &= !(1 << n);
        let pc = self.regs.pc;
        self.pushstack(pc);
        self.regs.pc = 0x0040 | ((n as u16) << 3);

        4
    }

    fn pushstack(&mut self, value: u16) {
        self.regs.sp = self.regs.sp.wrapping_sub(2);
        self.mmu.ww(self.regs.sp, value);
    }

    fn popstack(&mut self) -> u16 {
        let res = self.mmu.rw(self.regs.sp);
        self.regs.sp += 2;
        res
    }

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
            0x00 => 1,
            0x01 => {
                let v = self.fetchword();
                self.regs.setbc(v);
                3
            }
            0x02 => {
                self.mmu.wb(self.regs.bc(), self.regs.a);
                2
            }
            0x03 => {
                self.regs.setbc(self.regs.bc().wrapping_add(1));
                2
            }
            0x04 => {
                self.regs.b = self.alu_inc(self.regs.b);
                1
            }
            0x05 => {
                self.regs.b = self.alu_dec(self.regs.b);
                1
            }
            0x06 => {
                self.regs.b = self.fetch_byte();
                2
            }
            0x07 => {
                self.regs.a = self.alu_rlc(self.regs.a);
                self.regs.flag(Z, false);
                1
            }
            0x08 => {
                let a = self.fetchword();
                self.mmu.ww(a, self.regs.sp);
                5
            }
            0x09 => {
                self.alu_add16(self.regs.bc());
                2
            }
            0x0A => {
                self.regs.a = self.mmu.rb(self.regs.bc());
                2
            }
            0x0B => {
                self.regs.setbc(self.regs.bc().wrapping_sub(1));
                2
            }
            0x0C => {
                self.regs.c = self.alu_inc(self.regs.c);
                1
            }
            0x0D => {
                self.regs.c = self.alu_dec(self.regs.c);
                1
            }
            0x0E => {
                self.regs.c = self.fetch_byte();
                2
            }
            0x0F => {
                self.regs.a = self.alu_rrc(self.regs.a);
                self.regs.flag(Z, false);
                1
            }
            0x10 => {
                self.mmu.switch_speed();
                1
            } // STOP
            0x11 => {
                let v = self.fetchword();
                self.regs.setde(v);
                3
            }
            0x12 => {
                self.mmu.wb(self.regs.de(), self.regs.a);
                2
            }
            0x13 => {
                self.regs.setde(self.regs.de().wrapping_add(1));
                2
            }
            0x14 => {
                self.regs.d = self.alu_inc(self.regs.d);
                1
            }
            0x15 => {
                self.regs.d = self.alu_dec(self.regs.d);
                1
            }
            0x16 => {
                self.regs.d = self.fetch_byte();
                2
            }
            0x17 => {
                self.regs.a = self.alu_rl(self.regs.a);
                self.regs.flag(Z, false);
                1
            }
            0x18 => {
                self.cpu_jr();
                3
            }
            0x19 => {
                self.alu_add16(self.regs.de());
                2
            }
            0x1A => {
                self.regs.a = self.mmu.rb(self.regs.de());
                2
            }
            0x1B => {
                self.regs.setde(self.regs.de().wrapping_sub(1));
                2
            }
            0x1C => {
                self.regs.e = self.alu_inc(self.regs.e);
                1
            }
            0x1D => {
                self.regs.e = self.alu_dec(self.regs.e);
                1
            }
            0x1E => {
                self.regs.e = self.fetch_byte();
                2
            }
            0x1F => {
                self.regs.a = self.alu_rr(self.regs.a);
                self.regs.flag(Z, false);
                1
            }
            0x20 => {
                if !self.regs.getflag(Z) {
                    self.cpu_jr();
                    3
                } else {
                    self.regs.pc += 1;
                    2
                }
            }
            0x21 => {
                let v = self.fetchword();
                self.regs.sethl(v);
                3
            }
            0x22 => {
                self.mmu.wb(self.regs.hli(), self.regs.a);
                2
            }
            0x23 => {
                let v = self.regs.hl().wrapping_add(1);
                self.regs.sethl(v);
                2
            }
            0x24 => {
                self.regs.h = self.alu_inc(self.regs.h);
                1
            }
            0x25 => {
                self.regs.h = self.alu_dec(self.regs.h);
                1
            }
            0x26 => {
                self.regs.h = self.fetch_byte();
                2
            }
            0x27 => {
                self.alu_daa();
                1
            }
            0x28 => {
                if self.regs.getflag(Z) {
                    self.cpu_jr();
                    3
                } else {
                    self.regs.pc += 1;
                    2
                }
            }
            0x29 => {
                let v = self.regs.hl();
                self.alu_add16(v);
                2
            }
            0x2A => {
                self.regs.a = self.mmu.rb(self.regs.hli());
                2
            }
            0x2B => {
                let v = self.regs.hl().wrapping_sub(1);
                self.regs.sethl(v);
                2
            }
            0x2C => {
                self.regs.l = self.alu_inc(self.regs.l);
                1
            }
            0x2D => {
                self.regs.l = self.alu_dec(self.regs.l);
                1
            }
            0x2E => {
                self.regs.l = self.fetch_byte();
                2
            }
            0x2F => {
                self.regs.a = !self.regs.a;
                self.regs.flag(H, true);
                self.regs.flag(N, true);
                1
            }
            0x30 => {
                if !self.regs.getflag(C) {
                    self.cpu_jr();
                    3
                } else {
                    self.regs.pc += 1;
                    2
                }
            }
            0x31 => {
                self.regs.sp = self.fetchword();
                3
            }
            0x32 => {
                self.mmu.wb(self.regs.hld(), self.regs.a);
                2
            }
            0x33 => {
                self.regs.sp = self.regs.sp.wrapping_add(1);
                2
            }
            0x34 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a);
                let v2 = self.alu_inc(v);
                self.mmu.wb(a, v2);
                3
            }
            0x35 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a);
                let v2 = self.alu_dec(v);
                self.mmu.wb(a, v2);
                3
            }
            0x36 => {
                let v = self.fetch_byte();
                self.mmu.wb(self.regs.hl(), v);
                3
            }
            0x37 => {
                self.regs.flag(C, true);
                self.regs.flag(H, false);
                self.regs.flag(N, false);
                1
            }
            0x38 => {
                if self.regs.getflag(C) {
                    self.cpu_jr();
                    3
                } else {
                    self.regs.pc += 1;
                    2
                }
            }
            0x39 => {
                self.alu_add16(self.regs.sp);
                2
            }
            0x3A => {
                self.regs.a = self.mmu.rb(self.regs.hld());
                2
            }
            0x3B => {
                self.regs.sp = self.regs.sp.wrapping_sub(1);
                2
            }
            0x3C => {
                self.regs.a = self.alu_inc(self.regs.a);
                1
            }
            0x3D => {
                self.regs.a = self.alu_dec(self.regs.a);
                1
            }
            0x3E => {
                self.regs.a = self.fetch_byte();
                2
            }
            0x3F => {
                let v = !self.regs.getflag(C);
                self.regs.flag(C, v);
                self.regs.flag(H, false);
                self.regs.flag(N, false);
                1
            }
            0x40 => 1,
            0x41 => {
                self.regs.b = self.regs.c;
                1
            }
            0x42 => {
                self.regs.b = self.regs.d;
                1
            }
            0x43 => {
                self.regs.b = self.regs.e;
                1
            }
            0x44 => {
                self.regs.b = self.regs.h;
                1
            }
            0x45 => {
                self.regs.b = self.regs.l;
                1
            }
            0x46 => {
                self.regs.b = self.mmu.rb(self.regs.hl());
                2
            }
            0x47 => {
                self.regs.b = self.regs.a;
                1
            }
            0x48 => {
                self.regs.c = self.regs.b;
                1
            }
            0x49 => 1,
            0x4A => {
                self.regs.c = self.regs.d;
                1
            }
            0x4B => {
                self.regs.c = self.regs.e;
                1
            }
            0x4C => {
                self.regs.c = self.regs.h;
                1
            }
            0x4D => {
                self.regs.c = self.regs.l;
                1
            }
            0x4E => {
                self.regs.c = self.mmu.rb(self.regs.hl());
                2
            }
            0x4F => {
                self.regs.c = self.regs.a;
                1
            }
            0x50 => {
                self.regs.d = self.regs.b;
                1
            }
            0x51 => {
                self.regs.d = self.regs.c;
                1
            }
            0x52 => 1,
            0x53 => {
                self.regs.d = self.regs.e;
                1
            }
            0x54 => {
                self.regs.d = self.regs.h;
                1
            }
            0x55 => {
                self.regs.d = self.regs.l;
                1
            }
            0x56 => {
                self.regs.d = self.mmu.rb(self.regs.hl());
                2
            }
            0x57 => {
                self.regs.d = self.regs.a;
                1
            }
            0x58 => {
                self.regs.e = self.regs.b;
                1
            }
            0x59 => {
                self.regs.e = self.regs.c;
                1
            }
            0x5A => {
                self.regs.e = self.regs.d;
                1
            }
            0x5B => 1,
            0x5C => {
                self.regs.e = self.regs.h;
                1
            }
            0x5D => {
                self.regs.e = self.regs.l;
                1
            }
            0x5E => {
                self.regs.e = self.mmu.rb(self.regs.hl());
                2
            }
            0x5F => {
                self.regs.e = self.regs.a;
                1
            }
            0x60 => {
                self.regs.h = self.regs.b;
                1
            }
            0x61 => {
                self.regs.h = self.regs.c;
                1
            }
            0x62 => {
                self.regs.h = self.regs.d;
                1
            }
            0x63 => {
                self.regs.h = self.regs.e;
                1
            }
            0x64 => 1,
            0x65 => {
                self.regs.h = self.regs.l;
                1
            }
            0x66 => {
                self.regs.h = self.mmu.rb(self.regs.hl());
                2
            }
            0x67 => {
                self.regs.h = self.regs.a;
                1
            }
            0x68 => {
                self.regs.l = self.regs.b;
                1
            }
            0x69 => {
                self.regs.l = self.regs.c;
                1
            }
            0x6A => {
                self.regs.l = self.regs.d;
                1
            }
            0x6B => {
                self.regs.l = self.regs.e;
                1
            }
            0x6C => {
                self.regs.l = self.regs.h;
                1
            }
            0x6D => 1,
            0x6E => {
                self.regs.l = self.mmu.rb(self.regs.hl());
                2
            }
            0x6F => {
                self.regs.l = self.regs.a;
                1
            }
            0x70 => {
                self.mmu.wb(self.regs.hl(), self.regs.b);
                2
            }
            0x71 => {
                self.mmu.wb(self.regs.hl(), self.regs.c);
                2
            }
            0x72 => {
                self.mmu.wb(self.regs.hl(), self.regs.d);
                2
            }
            0x73 => {
                self.mmu.wb(self.regs.hl(), self.regs.e);
                2
            }
            0x74 => {
                self.mmu.wb(self.regs.hl(), self.regs.h);
                2
            }
            0x75 => {
                self.mmu.wb(self.regs.hl(), self.regs.l);
                2
            }
            0x76 => {
                self.halted = true;
                self.halt_bug = self.mmu.intf & self.mmu.inte & 0x1F != 0;
                1
            }
            0x77 => {
                self.mmu.wb(self.regs.hl(), self.regs.a);
                2
            }
            0x78 => {
                self.regs.a = self.regs.b;
                1
            }
            0x79 => {
                self.regs.a = self.regs.c;
                1
            }
            0x7A => {
                self.regs.a = self.regs.d;
                1
            }
            0x7B => {
                self.regs.a = self.regs.e;
                1
            }
            0x7C => {
                self.regs.a = self.regs.h;
                1
            }
            0x7D => {
                self.regs.a = self.regs.l;
                1
            }
            0x7E => {
                self.regs.a = self.mmu.rb(self.regs.hl());
                2
            }
            0x7F => 1,
            0x80 => {
                self.alu_add(self.regs.b, false);
                1
            }
            0x81 => {
                self.alu_add(self.regs.c, false);
                1
            }
            0x82 => {
                self.alu_add(self.regs.d, false);
                1
            }
            0x83 => {
                self.alu_add(self.regs.e, false);
                1
            }
            0x84 => {
                self.alu_add(self.regs.h, false);
                1
            }
            0x85 => {
                self.alu_add(self.regs.l, false);
                1
            }
            0x86 => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_add(v, false);
                2
            }
            0x87 => {
                self.alu_add(self.regs.a, false);
                1
            }
            0x88 => {
                self.alu_add(self.regs.b, true);
                1
            }
            0x89 => {
                self.alu_add(self.regs.c, true);
                1
            }
            0x8A => {
                self.alu_add(self.regs.d, true);
                1
            }
            0x8B => {
                self.alu_add(self.regs.e, true);
                1
            }
            0x8C => {
                self.alu_add(self.regs.h, true);
                1
            }
            0x8D => {
                self.alu_add(self.regs.l, true);
                1
            }
            0x8E => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_add(v, true);
                2
            }
            0x8F => {
                self.alu_add(self.regs.a, true);
                1
            }
            0x90 => {
                self.alu_sub(self.regs.b, false);
                1
            }
            0x91 => {
                self.alu_sub(self.regs.c, false);
                1
            }
            0x92 => {
                self.alu_sub(self.regs.d, false);
                1
            }
            0x93 => {
                self.alu_sub(self.regs.e, false);
                1
            }
            0x94 => {
                self.alu_sub(self.regs.h, false);
                1
            }
            0x95 => {
                self.alu_sub(self.regs.l, false);
                1
            }
            0x96 => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_sub(v, false);
                2
            }
            0x97 => {
                self.alu_sub(self.regs.a, false);
                1
            }
            0x98 => {
                self.alu_sub(self.regs.b, true);
                1
            }
            0x99 => {
                self.alu_sub(self.regs.c, true);
                1
            }
            0x9A => {
                self.alu_sub(self.regs.d, true);
                1
            }
            0x9B => {
                self.alu_sub(self.regs.e, true);
                1
            }
            0x9C => {
                self.alu_sub(self.regs.h, true);
                1
            }
            0x9D => {
                self.alu_sub(self.regs.l, true);
                1
            }
            0x9E => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_sub(v, true);
                2
            }
            0x9F => {
                self.alu_sub(self.regs.a, true);
                1
            }
            0xA0 => {
                self.alu_and(self.regs.b);
                1
            }
            0xA1 => {
                self.alu_and(self.regs.c);
                1
            }
            0xA2 => {
                self.alu_and(self.regs.d);
                1
            }
            0xA3 => {
                self.alu_and(self.regs.e);
                1
            }
            0xA4 => {
                self.alu_and(self.regs.h);
                1
            }
            0xA5 => {
                self.alu_and(self.regs.l);
                1
            }
            0xA6 => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_and(v);
                2
            }
            0xA7 => {
                self.alu_and(self.regs.a);
                1
            }
            0xA8 => {
                self.alu_xor(self.regs.b);
                1
            }
            0xA9 => {
                self.alu_xor(self.regs.c);
                1
            }
            0xAA => {
                self.alu_xor(self.regs.d);
                1
            }
            0xAB => {
                self.alu_xor(self.regs.e);
                1
            }
            0xAC => {
                self.alu_xor(self.regs.h);
                1
            }
            0xAD => {
                self.alu_xor(self.regs.l);
                1
            }
            0xAE => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_xor(v);
                2
            }
            0xAF => {
                self.alu_xor(self.regs.a);
                1
            }
            0xB0 => {
                self.alu_or(self.regs.b);
                1
            }
            0xB1 => {
                self.alu_or(self.regs.c);
                1
            }
            0xB2 => {
                self.alu_or(self.regs.d);
                1
            }
            0xB3 => {
                self.alu_or(self.regs.e);
                1
            }
            0xB4 => {
                self.alu_or(self.regs.h);
                1
            }
            0xB5 => {
                self.alu_or(self.regs.l);
                1
            }
            0xB6 => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_or(v);
                2
            }
            0xB7 => {
                self.alu_or(self.regs.a);
                1
            }
            0xB8 => {
                self.alu_cp(self.regs.b);
                1
            }
            0xB9 => {
                self.alu_cp(self.regs.c);
                1
            }
            0xBA => {
                self.alu_cp(self.regs.d);
                1
            }
            0xBB => {
                self.alu_cp(self.regs.e);
                1
            }
            0xBC => {
                self.alu_cp(self.regs.h);
                1
            }
            0xBD => {
                self.alu_cp(self.regs.l);
                1
            }
            0xBE => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_cp(v);
                2
            }
            0xBF => {
                self.alu_cp(self.regs.a);
                1
            }
            0xC0 => {
                if !self.regs.getflag(Z) {
                    self.regs.pc = self.popstack();
                    5
                } else {
                    2
                }
            }
            0xC1 => {
                let v = self.popstack();
                self.regs.setbc(v);
                3
            }
            0xC2 => {
                if !self.regs.getflag(Z) {
                    self.regs.pc = self.fetchword();
                    4
                } else {
                    self.regs.pc += 2;
                    3
                }
            }
            0xC3 => {
                self.regs.pc = self.fetchword();
                4
            }
            0xC4 => {
                if !self.regs.getflag(Z) {
                    self.pushstack(self.regs.pc + 2);
                    self.regs.pc = self.fetchword();
                    6
                } else {
                    self.regs.pc += 2;
                    3
                }
            }
            0xC5 => {
                self.pushstack(self.regs.bc());
                4
            }
            0xC6 => {
                let v = self.fetch_byte();
                self.alu_add(v, false);
                2
            }
            0xC7 => {
                self.pushstack(self.regs.pc);
                self.regs.pc = 0x00;
                4
            }
            0xC8 => {
                if self.regs.getflag(Z) {
                    self.regs.pc = self.popstack();
                    5
                } else {
                    2
                }
            }
            0xC9 => {
                self.regs.pc = self.popstack();
                4
            }
            0xCA => {
                if self.regs.getflag(Z) {
                    self.regs.pc = self.fetchword();
                    4
                } else {
                    self.regs.pc += 2;
                    3
                }
            }
            0xCB => self.call_cb(),
            0xCC => {
                if self.regs.getflag(Z) {
                    self.pushstack(self.regs.pc + 2);
                    self.regs.pc = self.fetchword();
                    6
                } else {
                    self.regs.pc += 2;
                    3
                }
            }
            0xCD => {
                self.pushstack(self.regs.pc + 2);
                self.regs.pc = self.fetchword();
                6
            }
            0xCE => {
                let v = self.fetch_byte();
                self.alu_add(v, true);
                2
            }
            0xCF => {
                self.pushstack(self.regs.pc);
                self.regs.pc = 0x08;
                4
            }
            0xD0 => {
                if !self.regs.getflag(C) {
                    self.regs.pc = self.popstack();
                    5
                } else {
                    2
                }
            }
            0xD1 => {
                let v = self.popstack();
                self.regs.setde(v);
                3
            }
            0xD2 => {
                if !self.regs.getflag(C) {
                    self.regs.pc = self.fetchword();
                    4
                } else {
                    self.regs.pc += 2;
                    3
                }
            }
            0xD4 => {
                if !self.regs.getflag(C) {
                    self.pushstack(self.regs.pc + 2);
                    self.regs.pc = self.fetchword();
                    6
                } else {
                    self.regs.pc += 2;
                    3
                }
            }
            0xD5 => {
                self.pushstack(self.regs.de());
                4
            }
            0xD6 => {
                let v = self.fetch_byte();
                self.alu_sub(v, false);
                2
            }
            0xD7 => {
                self.pushstack(self.regs.pc);
                self.regs.pc = 0x10;
                4
            }
            0xD8 => {
                if self.regs.getflag(C) {
                    self.regs.pc = self.popstack();
                    5
                } else {
                    2
                }
            }
            0xD9 => {
                self.regs.pc = self.popstack();
                self.setei = 1;
                4
            }
            0xDA => {
                if self.regs.getflag(C) {
                    self.regs.pc = self.fetchword();
                    4
                } else {
                    self.regs.pc += 2;
                    3
                }
            }
            0xDC => {
                if self.regs.getflag(C) {
                    self.pushstack(self.regs.pc + 2);
                    self.regs.pc = self.fetchword();
                    6
                } else {
                    self.regs.pc += 2;
                    3
                }
            }
            0xDE => {
                let v = self.fetch_byte();
                self.alu_sub(v, true);
                2
            }
            0xDF => {
                self.pushstack(self.regs.pc);
                self.regs.pc = 0x18;
                4
            }
            0xE0 => {
                let a = 0xFF00 | self.fetch_byte() as u16;
                self.mmu.wb(a, self.regs.a);
                3
            }
            0xE1 => {
                let v = self.popstack();
                self.regs.sethl(v);
                3
            }
            0xE2 => {
                self.mmu.wb(0xFF00 | self.regs.c as u16, self.regs.a);
                2
            }
            0xE5 => {
                self.pushstack(self.regs.hl());
                4
            }
            0xE6 => {
                let v = self.fetch_byte();
                self.alu_and(v);
                2
            }
            0xE7 => {
                self.pushstack(self.regs.pc);
                self.regs.pc = 0x20;
                4
            }
            0xE8 => {
                self.regs.sp = self.alu_add16imm(self.regs.sp);
                4
            }
            0xE9 => {
                self.regs.pc = self.regs.hl();
                1
            }
            0xEA => {
                let a = self.fetchword();
                self.mmu.wb(a, self.regs.a);
                4
            }
            0xEE => {
                let v = self.fetch_byte();
                self.alu_xor(v);
                2
            }
            0xEF => {
                self.pushstack(self.regs.pc);
                self.regs.pc = 0x28;
                4
            }
            0xF0 => {
                let a = 0xFF00 | self.fetch_byte() as u16;
                self.regs.a = self.mmu.rb(a);
                3
            }
            0xF1 => {
                let v = self.popstack() & 0xFFF0;
                self.regs.setaf(v);
                3
            }
            0xF2 => {
                self.regs.a = self.mmu.rb(0xFF00 | self.regs.c as u16);
                2
            }
            0xF3 => {
                self.setdi = 2;
                1
            }
            0xF5 => {
                self.pushstack(self.regs.af());
                4
            }
            0xF6 => {
                let v = self.fetch_byte();
                self.alu_or(v);
                2
            }
            0xF7 => {
                self.pushstack(self.regs.pc);
                self.regs.pc = 0x30;
                4
            }
            0xF8 => {
                let r = self.alu_add16imm(self.regs.sp);
                self.regs.sethl(r);
                3
            }
            0xF9 => {
                self.regs.sp = self.regs.hl();
                2
            }
            0xFA => {
                let a = self.fetchword();
                self.regs.a = self.mmu.rb(a);
                4
            }
            0xFB => {
                self.setei = 2;
                1
            }
            0xFE => {
                let v = self.fetch_byte();
                self.alu_cp(v);
                2
            }
            0xFF => {
                self.pushstack(self.regs.pc);
                self.regs.pc = 0x38;
                4
            }
            other => panic!("Instruction {:2X} is not implemented", other),
        }
    }

    fn call_cb(&mut self) -> u32 {
        let opcode = self.fetch_byte();
        match opcode {
            0x00 => {
                self.regs.b = self.alu_rlc(self.regs.b);
                2
            }
            0x01 => {
                self.regs.c = self.alu_rlc(self.regs.c);
                2
            }
            0x02 => {
                self.regs.d = self.alu_rlc(self.regs.d);
                2
            }
            0x03 => {
                self.regs.e = self.alu_rlc(self.regs.e);
                2
            }
            0x04 => {
                self.regs.h = self.alu_rlc(self.regs.h);
                2
            }
            0x05 => {
                self.regs.l = self.alu_rlc(self.regs.l);
                2
            }
            0x06 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a);
                let v2 = self.alu_rlc(v);
                self.mmu.wb(a, v2);
                4
            }
            0x07 => {
                self.regs.a = self.alu_rlc(self.regs.a);
                2
            }
            0x08 => {
                self.regs.b = self.alu_rrc(self.regs.b);
                2
            }
            0x09 => {
                self.regs.c = self.alu_rrc(self.regs.c);
                2
            }
            0x0A => {
                self.regs.d = self.alu_rrc(self.regs.d);
                2
            }
            0x0B => {
                self.regs.e = self.alu_rrc(self.regs.e);
                2
            }
            0x0C => {
                self.regs.h = self.alu_rrc(self.regs.h);
                2
            }
            0x0D => {
                self.regs.l = self.alu_rrc(self.regs.l);
                2
            }
            0x0E => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a);
                let v2 = self.alu_rrc(v);
                self.mmu.wb(a, v2);
                4
            }
            0x0F => {
                self.regs.a = self.alu_rrc(self.regs.a);
                2
            }
            0x10 => {
                self.regs.b = self.alu_rl(self.regs.b);
                2
            }
            0x11 => {
                self.regs.c = self.alu_rl(self.regs.c);
                2
            }
            0x12 => {
                self.regs.d = self.alu_rl(self.regs.d);
                2
            }
            0x13 => {
                self.regs.e = self.alu_rl(self.regs.e);
                2
            }
            0x14 => {
                self.regs.h = self.alu_rl(self.regs.h);
                2
            }
            0x15 => {
                self.regs.l = self.alu_rl(self.regs.l);
                2
            }
            0x16 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a);
                let v2 = self.alu_rl(v);
                self.mmu.wb(a, v2);
                4
            }
            0x17 => {
                self.regs.a = self.alu_rl(self.regs.a);
                2
            }
            0x18 => {
                self.regs.b = self.alu_rr(self.regs.b);
                2
            }
            0x19 => {
                self.regs.c = self.alu_rr(self.regs.c);
                2
            }
            0x1A => {
                self.regs.d = self.alu_rr(self.regs.d);
                2
            }
            0x1B => {
                self.regs.e = self.alu_rr(self.regs.e);
                2
            }
            0x1C => {
                self.regs.h = self.alu_rr(self.regs.h);
                2
            }
            0x1D => {
                self.regs.l = self.alu_rr(self.regs.l);
                2
            }
            0x1E => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a);
                let v2 = self.alu_rr(v);
                self.mmu.wb(a, v2);
                4
            }
            0x1F => {
                self.regs.a = self.alu_rr(self.regs.a);
                2
            }
            0x20 => {
                self.regs.b = self.alu_sla(self.regs.b);
                2
            }
            0x21 => {
                self.regs.c = self.alu_sla(self.regs.c);
                2
            }
            0x22 => {
                self.regs.d = self.alu_sla(self.regs.d);
                2
            }
            0x23 => {
                self.regs.e = self.alu_sla(self.regs.e);
                2
            }
            0x24 => {
                self.regs.h = self.alu_sla(self.regs.h);
                2
            }
            0x25 => {
                self.regs.l = self.alu_sla(self.regs.l);
                2
            }
            0x26 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a);
                let v2 = self.alu_sla(v);
                self.mmu.wb(a, v2);
                4
            }
            0x27 => {
                self.regs.a = self.alu_sla(self.regs.a);
                2
            }
            0x28 => {
                self.regs.b = self.alu_sra(self.regs.b);
                2
            }
            0x29 => {
                self.regs.c = self.alu_sra(self.regs.c);
                2
            }
            0x2A => {
                self.regs.d = self.alu_sra(self.regs.d);
                2
            }
            0x2B => {
                self.regs.e = self.alu_sra(self.regs.e);
                2
            }
            0x2C => {
                self.regs.h = self.alu_sra(self.regs.h);
                2
            }
            0x2D => {
                self.regs.l = self.alu_sra(self.regs.l);
                2
            }
            0x2E => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a);
                let v2 = self.alu_sra(v);
                self.mmu.wb(a, v2);
                4
            }
            0x2F => {
                self.regs.a = self.alu_sra(self.regs.a);
                2
            }
            0x30 => {
                self.regs.b = self.alu_swap(self.regs.b);
                2
            }
            0x31 => {
                self.regs.c = self.alu_swap(self.regs.c);
                2
            }
            0x32 => {
                self.regs.d = self.alu_swap(self.regs.d);
                2
            }
            0x33 => {
                self.regs.e = self.alu_swap(self.regs.e);
                2
            }
            0x34 => {
                self.regs.h = self.alu_swap(self.regs.h);
                2
            }
            0x35 => {
                self.regs.l = self.alu_swap(self.regs.l);
                2
            }
            0x36 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a);
                let v2 = self.alu_swap(v);
                self.mmu.wb(a, v2);
                4
            }
            0x37 => {
                self.regs.a = self.alu_swap(self.regs.a);
                2
            }
            0x38 => {
                self.regs.b = self.alu_srl(self.regs.b);
                2
            }
            0x39 => {
                self.regs.c = self.alu_srl(self.regs.c);
                2
            }
            0x3A => {
                self.regs.d = self.alu_srl(self.regs.d);
                2
            }
            0x3B => {
                self.regs.e = self.alu_srl(self.regs.e);
                2
            }
            0x3C => {
                self.regs.h = self.alu_srl(self.regs.h);
                2
            }
            0x3D => {
                self.regs.l = self.alu_srl(self.regs.l);
                2
            }
            0x3E => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a);
                let v2 = self.alu_srl(v);
                self.mmu.wb(a, v2);
                4
            }
            0x3F => {
                self.regs.a = self.alu_srl(self.regs.a);
                2
            }
            0x40 => {
                self.alu_bit(self.regs.b, 0);
                2
            }
            0x41 => {
                self.alu_bit(self.regs.c, 0);
                2
            }
            0x42 => {
                self.alu_bit(self.regs.d, 0);
                2
            }
            0x43 => {
                self.alu_bit(self.regs.e, 0);
                2
            }
            0x44 => {
                self.alu_bit(self.regs.h, 0);
                2
            }
            0x45 => {
                self.alu_bit(self.regs.l, 0);
                2
            }
            0x46 => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_bit(v, 0);
                3
            }
            0x47 => {
                self.alu_bit(self.regs.a, 0);
                2
            }
            0x48 => {
                self.alu_bit(self.regs.b, 1);
                2
            }
            0x49 => {
                self.alu_bit(self.regs.c, 1);
                2
            }
            0x4A => {
                self.alu_bit(self.regs.d, 1);
                2
            }
            0x4B => {
                self.alu_bit(self.regs.e, 1);
                2
            }
            0x4C => {
                self.alu_bit(self.regs.h, 1);
                2
            }
            0x4D => {
                self.alu_bit(self.regs.l, 1);
                2
            }
            0x4E => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_bit(v, 1);
                3
            }
            0x4F => {
                self.alu_bit(self.regs.a, 1);
                2
            }
            0x50 => {
                self.alu_bit(self.regs.b, 2);
                2
            }
            0x51 => {
                self.alu_bit(self.regs.c, 2);
                2
            }
            0x52 => {
                self.alu_bit(self.regs.d, 2);
                2
            }
            0x53 => {
                self.alu_bit(self.regs.e, 2);
                2
            }
            0x54 => {
                self.alu_bit(self.regs.h, 2);
                2
            }
            0x55 => {
                self.alu_bit(self.regs.l, 2);
                2
            }
            0x56 => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_bit(v, 2);
                3
            }
            0x57 => {
                self.alu_bit(self.regs.a, 2);
                2
            }
            0x58 => {
                self.alu_bit(self.regs.b, 3);
                2
            }
            0x59 => {
                self.alu_bit(self.regs.c, 3);
                2
            }
            0x5A => {
                self.alu_bit(self.regs.d, 3);
                2
            }
            0x5B => {
                self.alu_bit(self.regs.e, 3);
                2
            }
            0x5C => {
                self.alu_bit(self.regs.h, 3);
                2
            }
            0x5D => {
                self.alu_bit(self.regs.l, 3);
                2
            }
            0x5E => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_bit(v, 3);
                3
            }
            0x5F => {
                self.alu_bit(self.regs.a, 3);
                2
            }
            0x60 => {
                self.alu_bit(self.regs.b, 4);
                2
            }
            0x61 => {
                self.alu_bit(self.regs.c, 4);
                2
            }
            0x62 => {
                self.alu_bit(self.regs.d, 4);
                2
            }
            0x63 => {
                self.alu_bit(self.regs.e, 4);
                2
            }
            0x64 => {
                self.alu_bit(self.regs.h, 4);
                2
            }
            0x65 => {
                self.alu_bit(self.regs.l, 4);
                2
            }
            0x66 => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_bit(v, 4);
                3
            }
            0x67 => {
                self.alu_bit(self.regs.a, 4);
                2
            }
            0x68 => {
                self.alu_bit(self.regs.b, 5);
                2
            }
            0x69 => {
                self.alu_bit(self.regs.c, 5);
                2
            }
            0x6A => {
                self.alu_bit(self.regs.d, 5);
                2
            }
            0x6B => {
                self.alu_bit(self.regs.e, 5);
                2
            }
            0x6C => {
                self.alu_bit(self.regs.h, 5);
                2
            }
            0x6D => {
                self.alu_bit(self.regs.l, 5);
                2
            }
            0x6E => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_bit(v, 5);
                3
            }
            0x6F => {
                self.alu_bit(self.regs.a, 5);
                2
            }
            0x70 => {
                self.alu_bit(self.regs.b, 6);
                2
            }
            0x71 => {
                self.alu_bit(self.regs.c, 6);
                2
            }
            0x72 => {
                self.alu_bit(self.regs.d, 6);
                2
            }
            0x73 => {
                self.alu_bit(self.regs.e, 6);
                2
            }
            0x74 => {
                self.alu_bit(self.regs.h, 6);
                2
            }
            0x75 => {
                self.alu_bit(self.regs.l, 6);
                2
            }
            0x76 => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_bit(v, 6);
                3
            }
            0x77 => {
                self.alu_bit(self.regs.a, 6);
                2
            }
            0x78 => {
                self.alu_bit(self.regs.b, 7);
                2
            }
            0x79 => {
                self.alu_bit(self.regs.c, 7);
                2
            }
            0x7A => {
                self.alu_bit(self.regs.d, 7);
                2
            }
            0x7B => {
                self.alu_bit(self.regs.e, 7);
                2
            }
            0x7C => {
                self.alu_bit(self.regs.h, 7);
                2
            }
            0x7D => {
                self.alu_bit(self.regs.l, 7);
                2
            }
            0x7E => {
                let v = self.mmu.rb(self.regs.hl());
                self.alu_bit(v, 7);
                3
            }
            0x7F => {
                self.alu_bit(self.regs.a, 7);
                2
            }
            0x80 => {
                self.regs.b = self.regs.b & !(1 << 0);
                2
            }
            0x81 => {
                self.regs.c = self.regs.c & !(1 << 0);
                2
            }
            0x82 => {
                self.regs.d = self.regs.d & !(1 << 0);
                2
            }
            0x83 => {
                self.regs.e = self.regs.e & !(1 << 0);
                2
            }
            0x84 => {
                self.regs.h = self.regs.h & !(1 << 0);
                2
            }
            0x85 => {
                self.regs.l = self.regs.l & !(1 << 0);
                2
            }
            0x86 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) & !(1 << 0);
                self.mmu.wb(a, v);
                4
            }
            0x87 => {
                self.regs.a = self.regs.a & !(1 << 0);
                2
            }
            0x88 => {
                self.regs.b = self.regs.b & !(1 << 1);
                2
            }
            0x89 => {
                self.regs.c = self.regs.c & !(1 << 1);
                2
            }
            0x8A => {
                self.regs.d = self.regs.d & !(1 << 1);
                2
            }
            0x8B => {
                self.regs.e = self.regs.e & !(1 << 1);
                2
            }
            0x8C => {
                self.regs.h = self.regs.h & !(1 << 1);
                2
            }
            0x8D => {
                self.regs.l = self.regs.l & !(1 << 1);
                2
            }
            0x8E => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) & !(1 << 1);
                self.mmu.wb(a, v);
                4
            }
            0x8F => {
                self.regs.a = self.regs.a & !(1 << 1);
                2
            }
            0x90 => {
                self.regs.b = self.regs.b & !(1 << 2);
                2
            }
            0x91 => {
                self.regs.c = self.regs.c & !(1 << 2);
                2
            }
            0x92 => {
                self.regs.d = self.regs.d & !(1 << 2);
                2
            }
            0x93 => {
                self.regs.e = self.regs.e & !(1 << 2);
                2
            }
            0x94 => {
                self.regs.h = self.regs.h & !(1 << 2);
                2
            }
            0x95 => {
                self.regs.l = self.regs.l & !(1 << 2);
                2
            }
            0x96 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) & !(1 << 2);
                self.mmu.wb(a, v);
                4
            }
            0x97 => {
                self.regs.a = self.regs.a & !(1 << 2);
                2
            }
            0x98 => {
                self.regs.b = self.regs.b & !(1 << 3);
                2
            }
            0x99 => {
                self.regs.c = self.regs.c & !(1 << 3);
                2
            }
            0x9A => {
                self.regs.d = self.regs.d & !(1 << 3);
                2
            }
            0x9B => {
                self.regs.e = self.regs.e & !(1 << 3);
                2
            }
            0x9C => {
                self.regs.h = self.regs.h & !(1 << 3);
                2
            }
            0x9D => {
                self.regs.l = self.regs.l & !(1 << 3);
                2
            }
            0x9E => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) & !(1 << 3);
                self.mmu.wb(a, v);
                4
            }
            0x9F => {
                self.regs.a = self.regs.a & !(1 << 3);
                2
            }
            0xA0 => {
                self.regs.b = self.regs.b & !(1 << 4);
                2
            }
            0xA1 => {
                self.regs.c = self.regs.c & !(1 << 4);
                2
            }
            0xA2 => {
                self.regs.d = self.regs.d & !(1 << 4);
                2
            }
            0xA3 => {
                self.regs.e = self.regs.e & !(1 << 4);
                2
            }
            0xA4 => {
                self.regs.h = self.regs.h & !(1 << 4);
                2
            }
            0xA5 => {
                self.regs.l = self.regs.l & !(1 << 4);
                2
            }
            0xA6 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) & !(1 << 4);
                self.mmu.wb(a, v);
                4
            }
            0xA7 => {
                self.regs.a = self.regs.a & !(1 << 4);
                2
            }
            0xA8 => {
                self.regs.b = self.regs.b & !(1 << 5);
                2
            }
            0xA9 => {
                self.regs.c = self.regs.c & !(1 << 5);
                2
            }
            0xAA => {
                self.regs.d = self.regs.d & !(1 << 5);
                2
            }
            0xAB => {
                self.regs.e = self.regs.e & !(1 << 5);
                2
            }
            0xAC => {
                self.regs.h = self.regs.h & !(1 << 5);
                2
            }
            0xAD => {
                self.regs.l = self.regs.l & !(1 << 5);
                2
            }
            0xAE => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) & !(1 << 5);
                self.mmu.wb(a, v);
                4
            }
            0xAF => {
                self.regs.a = self.regs.a & !(1 << 5);
                2
            }
            0xB0 => {
                self.regs.b = self.regs.b & !(1 << 6);
                2
            }
            0xB1 => {
                self.regs.c = self.regs.c & !(1 << 6);
                2
            }
            0xB2 => {
                self.regs.d = self.regs.d & !(1 << 6);
                2
            }
            0xB3 => {
                self.regs.e = self.regs.e & !(1 << 6);
                2
            }
            0xB4 => {
                self.regs.h = self.regs.h & !(1 << 6);
                2
            }
            0xB5 => {
                self.regs.l = self.regs.l & !(1 << 6);
                2
            }
            0xB6 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) & !(1 << 6);
                self.mmu.wb(a, v);
                4
            }
            0xB7 => {
                self.regs.a = self.regs.a & !(1 << 6);
                2
            }
            0xB8 => {
                self.regs.b = self.regs.b & !(1 << 7);
                2
            }
            0xB9 => {
                self.regs.c = self.regs.c & !(1 << 7);
                2
            }
            0xBA => {
                self.regs.d = self.regs.d & !(1 << 7);
                2
            }
            0xBB => {
                self.regs.e = self.regs.e & !(1 << 7);
                2
            }
            0xBC => {
                self.regs.h = self.regs.h & !(1 << 7);
                2
            }
            0xBD => {
                self.regs.l = self.regs.l & !(1 << 7);
                2
            }
            0xBE => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) & !(1 << 7);
                self.mmu.wb(a, v);
                4
            }
            0xBF => {
                self.regs.a = self.regs.a & !(1 << 7);
                2
            }
            0xC0 => {
                self.regs.b = self.regs.b | (1 << 0);
                2
            }
            0xC1 => {
                self.regs.c = self.regs.c | (1 << 0);
                2
            }
            0xC2 => {
                self.regs.d = self.regs.d | (1 << 0);
                2
            }
            0xC3 => {
                self.regs.e = self.regs.e | (1 << 0);
                2
            }
            0xC4 => {
                self.regs.h = self.regs.h | (1 << 0);
                2
            }
            0xC5 => {
                self.regs.l = self.regs.l | (1 << 0);
                2
            }
            0xC6 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) | (1 << 0);
                self.mmu.wb(a, v);
                4
            }
            0xC7 => {
                self.regs.a = self.regs.a | (1 << 0);
                2
            }
            0xC8 => {
                self.regs.b = self.regs.b | (1 << 1);
                2
            }
            0xC9 => {
                self.regs.c = self.regs.c | (1 << 1);
                2
            }
            0xCA => {
                self.regs.d = self.regs.d | (1 << 1);
                2
            }
            0xCB => {
                self.regs.e = self.regs.e | (1 << 1);
                2
            }
            0xCC => {
                self.regs.h = self.regs.h | (1 << 1);
                2
            }
            0xCD => {
                self.regs.l = self.regs.l | (1 << 1);
                2
            }
            0xCE => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) | (1 << 1);
                self.mmu.wb(a, v);
                4
            }
            0xCF => {
                self.regs.a = self.regs.a | (1 << 1);
                2
            }
            0xD0 => {
                self.regs.b = self.regs.b | (1 << 2);
                2
            }
            0xD1 => {
                self.regs.c = self.regs.c | (1 << 2);
                2
            }
            0xD2 => {
                self.regs.d = self.regs.d | (1 << 2);
                2
            }
            0xD3 => {
                self.regs.e = self.regs.e | (1 << 2);
                2
            }
            0xD4 => {
                self.regs.h = self.regs.h | (1 << 2);
                2
            }
            0xD5 => {
                self.regs.l = self.regs.l | (1 << 2);
                2
            }
            0xD6 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) | (1 << 2);
                self.mmu.wb(a, v);
                4
            }
            0xD7 => {
                self.regs.a = self.regs.a | (1 << 2);
                2
            }
            0xD8 => {
                self.regs.b = self.regs.b | (1 << 3);
                2
            }
            0xD9 => {
                self.regs.c = self.regs.c | (1 << 3);
                2
            }
            0xDA => {
                self.regs.d = self.regs.d | (1 << 3);
                2
            }
            0xDB => {
                self.regs.e = self.regs.e | (1 << 3);
                2
            }
            0xDC => {
                self.regs.h = self.regs.h | (1 << 3);
                2
            }
            0xDD => {
                self.regs.l = self.regs.l | (1 << 3);
                2
            }
            0xDE => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) | (1 << 3);
                self.mmu.wb(a, v);
                4
            }
            0xDF => {
                self.regs.a = self.regs.a | (1 << 3);
                2
            }
            0xE0 => {
                self.regs.b = self.regs.b | (1 << 4);
                2
            }
            0xE1 => {
                self.regs.c = self.regs.c | (1 << 4);
                2
            }
            0xE2 => {
                self.regs.d = self.regs.d | (1 << 4);
                2
            }
            0xE3 => {
                self.regs.e = self.regs.e | (1 << 4);
                2
            }
            0xE4 => {
                self.regs.h = self.regs.h | (1 << 4);
                2
            }
            0xE5 => {
                self.regs.l = self.regs.l | (1 << 4);
                2
            }
            0xE6 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) | (1 << 4);
                self.mmu.wb(a, v);
                4
            }
            0xE7 => {
                self.regs.a = self.regs.a | (1 << 4);
                2
            }
            0xE8 => {
                self.regs.b = self.regs.b | (1 << 5);
                2
            }
            0xE9 => {
                self.regs.c = self.regs.c | (1 << 5);
                2
            }
            0xEA => {
                self.regs.d = self.regs.d | (1 << 5);
                2
            }
            0xEB => {
                self.regs.e = self.regs.e | (1 << 5);
                2
            }
            0xEC => {
                self.regs.h = self.regs.h | (1 << 5);
                2
            }
            0xED => {
                self.regs.l = self.regs.l | (1 << 5);
                2
            }
            0xEE => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) | (1 << 5);
                self.mmu.wb(a, v);
                4
            }
            0xEF => {
                self.regs.a = self.regs.a | (1 << 5);
                2
            }
            0xF0 => {
                self.regs.b = self.regs.b | (1 << 6);
                2
            }
            0xF1 => {
                self.regs.c = self.regs.c | (1 << 6);
                2
            }
            0xF2 => {
                self.regs.d = self.regs.d | (1 << 6);
                2
            }
            0xF3 => {
                self.regs.e = self.regs.e | (1 << 6);
                2
            }
            0xF4 => {
                self.regs.h = self.regs.h | (1 << 6);
                2
            }
            0xF5 => {
                self.regs.l = self.regs.l | (1 << 6);
                2
            }
            0xF6 => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) | (1 << 6);
                self.mmu.wb(a, v);
                4
            }
            0xF7 => {
                self.regs.a = self.regs.a | (1 << 6);
                2
            }
            0xF8 => {
                self.regs.b = self.regs.b | (1 << 7);
                2
            }
            0xF9 => {
                self.regs.c = self.regs.c | (1 << 7);
                2
            }
            0xFA => {
                self.regs.d = self.regs.d | (1 << 7);
                2
            }
            0xFB => {
                self.regs.e = self.regs.e | (1 << 7);
                2
            }
            0xFC => {
                self.regs.h = self.regs.h | (1 << 7);
                2
            }
            0xFD => {
                self.regs.l = self.regs.l | (1 << 7);
                2
            }
            0xFE => {
                let a = self.regs.hl();
                let v = self.mmu.rb(a) | (1 << 7);
                self.mmu.wb(a, v);
                4
            }
            0xFF => {
                self.regs.a = self.regs.a | (1 << 7);
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

    fn alu_add16imm(&mut self, a: u16) -> u16 {
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

    fn alu_rlc(&mut self, a: u8) -> u8 {
        let c = a & 0x80 == 0x80;
        let r = (a << 1) | (if c { 0x01 } else { 0x00 });
        self.alu_srflag_update(r, c);
        r
    }

    fn alu_rl(&mut self, a: u8) -> u8 {
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

    fn alu_rr(&mut self, a: u8) -> u8 {
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
        let c = a & 0x01 == 0x01;
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

        let mut adjust = if self.regs.get_flag(C) { 0x60 } else { 0x00 };
        if self.regs.get_flag(H) {
            adjust |= 0x06;
        }
        if !self.regs.get_flag(N) {
            if a & 0x0F > 0x09 {
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
