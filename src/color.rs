use smart_leds::RGB8;

pub struct Rainbow {
    pos: u8,
    step: u8,
}

impl Default for Rainbow {
    fn default() -> Self {
        Rainbow { pos: 0, step: 1 }
    }
}

impl Rainbow {
    pub fn step(step: u8) -> Self {
        Rainbow { pos: 0, step }
    }
    pub fn step_phase(step: u8, pos: u8) -> Self {
        Rainbow { pos, step }
    }
}

impl Iterator for Rainbow {
    type Item = RGB8;

    fn next(&mut self) -> Option<Self::Item> {
        let c = wheel(self.pos);
        self.pos = self.pos.overflowing_add(self.step).0;
        Some(c)
    }
}
/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
pub fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

pub const BLACK: RGB8 = RGB8 { r: 0, g: 0, b: 0 };
pub const RED: RGB8 = RGB8 { r: 255, g: 0, b: 0 };
pub const GREEN: RGB8 = RGB8 { r: 0, g: 255, b: 0 };
pub const BLUE: RGB8 = RGB8 { r: 0, g: 0, b: 255 };
pub const CYAN: RGB8 = RGB8 {
    r: 0,
    g: 255,
    b: 255,
};
pub const MAGENTA: RGB8 = RGB8 {
    r: 255,
    g: 0,
    b: 255,
};
pub const YELLOW: RGB8 = RGB8 {
    r: 255,
    g: 255,
    b: 0,
};
pub const WHITE: RGB8 = RGB8 {
    r: 255,
    g: 255,
    b: 255,
};
