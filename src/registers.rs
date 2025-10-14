use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

#[derive(Copy, Clone)]
pub enum CpuFlag {
    C = 0b00010000,
    H = 0b00100000,
    N = 0b01000000,
    Z = 0b10000000,
}

impl Registers {
    pub fn new() -> Registers {
        use CpuFlag::*;
        Registers {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            f: C as u8 | H as u8 | Z as u8,
            h: 0x01,
            l: 0x4D,
            pc: 0x0100,
            sp: 0xFFFE,
        }
    }

    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | ((self.f & 0xF0) as u16)
    }

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | ((self.c & 0xF0) as u16)
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | ((self.e & 0xF0) as u16)
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | ((self.l & 0xF0) as u16)
    }

    pub fn hli(&mut self) -> u16 {
        let res = self.hl();
        self.sethl(res + 1);
        res
    }

    pub fn setaf(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = (value & 0x00FF) as u8;
    }

    pub fn setbc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }

    pub fn setde(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }

    pub fn sethl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
    }

    pub fn flag(&mut self, flags: CpuFlag, set: bool) {
        let mask = flags as u8;
        match set {
            true => self.f |= mask,
            false => self.f &= !mask,
        }
        self.f &= 0xF0;
    }

    pub fn get_flag(&self, flags: CpuFlag) -> bool {
        let mask = flags as u8;
        self.f & mask > 0
    }

    pub fn setf(&mut self, flags: u8) {
        self.f = flags & 0xF0;
    }
}
