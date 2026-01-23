const VRAM_BANK_SIZE: usize = 0x2000; // 8KB
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;

const OAM_SIZE: usize = 0xA0; // 160B
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub struct Ppu {
    vram: [[u8; VRAM_BANK_SIZE]; 2],
    vram_bank: u8,
    oam: [u8; OAM_SIZE],
    lcdc: u8, // e/d display and controls background and sprite rendering.
    stat: u8, // status of interrupts e/d.
    scx: u8,  // control the scroll position x of the background.
    scy: u8,  // control the scroll position y of the background.
    ly: u8,   // current line
    lyc: u8, // controller that check ly value and trigger interrupt if stat bit2 == 1 then interrupt occur (stat bit6)
    dma: u8,
    bgp: u8, // Background palette data for assigning colors to tiles.
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,
    mode: u8, // tells what's the ppu doing.
    mode_clock: u32,
    pub interrupt: u8,
    framebuffer: Vec<u32>,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            vram: [[0; VRAM_BANK_SIZE]; 2],
            vram_bank: 1,
            oam: [0; OAM_SIZE],
            lcdc: 0,
            stat: 0,
            scx: 0,
            scy: 0,
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
            framebuffer: vec![0u32; SCREEN_HEIGHT * SCREEN_WIDTH],
        }
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

    pub fn get_framebuffer(&mut self) -> &Vec<u32> {
        &self.framebuffer
    }

    pub fn do_cycle(&mut self, ticks: u32) -> u32 {
        self.mode_clock += ticks;

        match self.mode {
            0 => { /*HBlank */ }
            1 => { /*VBlank */ }
            2 => { /* OAM search */ }
            3 => {
                self.draw_scanline();
            }
            _ => {}
        }
        0
    }

    fn draw_scanline(&mut self) {
        if self.lcdc & 0x80 != 0 {}
    }

    fn draw_sprite(&mut self, sprite_index: usize) {
        let sprite_address = sprite_index * 4;
        let x = (self.oam[sprite_address] - 8) as usize;
        let y = (self.oam[sprite_address] - 16) as usize;
        let tile_index = self.oam[sprite_address + 2] as usize;
        let attributes = self.oam[sprite_address + 3];

        let mut tile_pixels = self.fetch_tile(tile_index);
        if (attributes & 0x20) != 0 {
            flip_vertical(&mut tile_pixels);
        }
        if (attributes & 0x40) != 0 {
            flip_horizontal(&mut tile_pixels);
        }

        self.draw_tile(tile_pixels, x, y);
    }

    fn fetch_tile(&self, tile_index: usize) -> Vec<u32> {
        let tile_address = tile_index * 16;
        let mut tile_pixels = Vec::new();

        for row in 0..8 {
            let byte1 = self.vram[self.vram_bank as usize][tile_address + row * 2];
            let byte2 = self.vram[self.vram_bank as usize][tile_address + row * 2 + 1];

            for col in 0..8 {
                let bit1 = (byte1 as u32 >> (7 - col)) & 1;
                let bit2 = (byte2 as u32 >> (7 - col)) & 1;
                let color = (bit2 << 1) | bit1;
                tile_pixels.push(color);
            }
        }

        tile_pixels
    }

    fn draw_tile(&mut self, tile_pixels: Vec<u32>, x: usize, y: usize) {
        for row in 0..8 {
            for col in 0..8 {
                let color = tile_pixels[row * 8 + col];
                self.framebuffer[(y + row) * SCREEN_WIDTH + (x + col)] = convert_color(color)
            }
        }
    }
}

fn convert_color(color: u32) -> u32 {
    match color {
        0 => 0xFFFFFFFF,
        1 => 0xFFAAAAAA,
        2 => 0xFF555555,
        3 => 0xFF000000,
        _ => 0xFFFFFFFF,
    }
}

fn flip_vertical(tile_pixel: &mut [u32]) {
    for row in 0..4 {
        let start = row * 8;
        let end = (7 - row) * 8;

        for col in 0..8 {
            tile_pixel.swap(start + col, end + col);
        }
    }
}

fn flip_horizontal(tile_pixel: &mut [u32]) {
    for row in 0..4 {
        let start = row * 8;

        for col in 0..8 {
            tile_pixel.swap(start + col, start + 7 - col);
        }
    }
}
