use winit::keyboard::Key;

pub struct Joypad {
    action: bool,
    direction: bool,
    up: bool,
    down: bool,
    right: bool,
    left: bool,
    a: bool,
    b: bool,
    start: bool,
    select: bool,
    pub inte: u8,
}

#[derive(Clone, Copy)]
pub enum KeypadKey {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            action: false,
            direction: false,
            up: false,
            down: false,
            right: false,
            left: false,
            a: false,
            b: false,
            start: false,
            select: false,
            inte: 0,
        }
    }

    pub fn rb(&self) -> u8 {
        let mut res = 0xC0;

        if self.action {
            res |= 0x20;
        }
        if self.direction {
            res |= 0x10;
        }

        if !self.direction {
            if self.right {
                res &= !0x01;
            }
            if self.left {
                res &= !0x02;
            }
            if self.up {
                res &= !0x04;
            }
            if self.down {
                res &= !0x08;
            }
        }

        if !self.action {
            if self.a {
                res &= !0x01;
            }
            if self.b {
                res &= !0x02;
            }
            if self.select {
                res &= !0x04;
            }
            if self.start {
                res &= !0x08;
            }
        }

        res
    }

    pub fn wb(&mut self, v: u8) {
        self.action = v & 0x20 != 0;
        self.direction = v & 0x10 != 0;
    }

    pub fn press_button(&mut self, button: KeypadKey) {
        let was_pressed = self.any_button_pressed();

        match button {
            KeypadKey::Right => self.right = true,
            KeypadKey::Left => self.left = true,
            KeypadKey::Up => self.up = true,
            KeypadKey::Down => self.down = true,
            KeypadKey::A => self.a = true,
            KeypadKey::B => self.b = true,
            KeypadKey::Start => self.start = true,
            KeypadKey::Select => self.select = true,
        }

        if !was_pressed {
            self.inte |= 0x10
        }
    }

    pub fn release_button(&mut self, button: KeypadKey) {
        match button {
            KeypadKey::Right => self.right = false,
            KeypadKey::Left => self.left = false,
            KeypadKey::Up => self.up = false,
            KeypadKey::Down => self.down = false,
            KeypadKey::A => self.a = false,
            KeypadKey::B => self.b = false,
            KeypadKey::Start => self.start = false,
            KeypadKey::Select => self.select = false,
        }
    }

    fn any_button_pressed(&self) -> bool {
        self.right
            || self.left
            || self.up
            || self.down
            || self.a
            || self.b
            || self.start
            || self.select
    }
}
