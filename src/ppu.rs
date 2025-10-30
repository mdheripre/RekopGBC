const VRAM_BANK_SIZE: usize = 0x2000; // 8KB
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;

const OAM_SIZE: usize = 0xA0; // 160B
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;

pub struct Ppu {
    vram: [[u8; VRAM_BANK_SIZE]; 2],
    vram_bank: u8,
    oam: [u8; OAM_SIZE],
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    dma: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,
    mode: u8,
    mode_clock: u32,
    pub interrupt: u8,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            vram: [[0; VRAM_BANK_SIZE]; 2],
            vram_bank: 1,
            oam: [0; OAM_SIZE],
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            dma: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,
            mode: 0,
            mode_clock: 0,
            interrupt: 0,
        }
    }

    pub fn get_framebuffer(&self) -> Vec<u32> {
        let mut framebuffer = vec![0u32; 160 * 144];

        for y in 0..144 {
            for x in 0..160 {
                let pixel_index = y * 160 + x;
                let r = (x % 255) as u8;
                let g = (y % 255) as u8;
                let b = ((x + y) % 255) as u8;

                framebuffer[pixel_index] =
                    (0xFF << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            }
        }

        framebuffer
    }

    pub fn do_cycle(&mut self, ticks: u32) -> u32 {
        0
    }

    pub fn rb(&self, a: u16) -> u8 {
        match a {
            0x8000..=0x9FFF => self.vram[self.vram_bank as usize][(a - 0x8000) as usize],
            0xFE00..=0xFE9F => self.oam[(a - 0xFE00) as usize],
            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF45 => self.lyc,
            0xFF46 => self.dma,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF,
        }
    }

    pub fn wb(&mut self, a: u16, v: u8) {
        match a {
            0x8000..=0x9FFF => self.vram[self.vram_bank as usize][(a - 0x8000) as usize] = v,
            0xFE00..=0xFE9F => self.oam[(a - 0xFE00) as usize] = v,
            0xFF40 => self.lcdc = v,
            0xFF41 => self.stat = v,
            0xFF42 => self.scy = v,
            0xFF43 => self.scx = v,
            0xFF44 => self.ly = 0,
            0xFF45 => self.lyc = v,
            0xFF46 => self.dma = v,
            0xFF47 => self.bgp = v,
            0xFF48 => self.obp0 = v,
            0xFF49 => self.obp1 = v,
            0xFF4A => self.wy = v,
            0xFF4B => self.wx = v,
            0xFF4F => self.vram_bank = v & 0x01,
            _ => panic!("Ppu error: cannot write {:4X}", a),
        }
    }
}
