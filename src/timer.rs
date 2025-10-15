use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Timer {
    tima: u8,
    tma: u8,
    tac: u8,
    div: u8,
    internal_div: u32,
    internal_counter: u32,
    interrupt: u8,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            tima: 0,
            tma: 0,
            tac: 0,
            div: 0,
            internal_div: 0,
            internal_counter: 0,
            interrupt: 0,
        }
    }

    pub fn rb(&mut self, a: u16) -> u8 {
        match a {
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0xF8,
            _ => panic!("Timer error: cannot read {:4X}", a),
        }
    }

    pub fn wb(&mut self, a: u16, v: u8) {
        match a {
            0xFF04 => self.div = 0,
            0xFF05 => self.tima = v,
            0xFF06 => self.tma = v,
            0xFF07 => self.tac = v,
            _ => panic!("Timer error: cannot write {:4X}", a),
        }
    }

    pub fn do_cycle(&mut self, ticks: u32) {
        self.internal_div += ticks;
        while self.internal_div >= 256 {
            self.div = self.div.wrapping_add(1);
            self.internal_div -= 256;
        }

        if self.tac & 0x04 != 0 {
            let step = match self.tac & 0x03 {
                0b00 => 1024,
                0b01 => 16,
                0b10 => 64,
                0b11 => 256,
                _=> unreachable!(),
            };

            self.internal_counter += ticks;
            while self.internal_counter >= step {
                self.tima = self.tima.wrapping_add(1);

                if self.tima == 0 {
                    self.tima = self.tma;
                    self.interrupt |= 0x04;
                }
                self.internal_counter -= step;
            }
        }
    }
}
