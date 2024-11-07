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

// simple hue/value color representation
#[derive(Clone, Copy)]
pub struct HV8 {
    pub h: u8,
    pub v: u8,
}

impl HV8 {
    pub fn zero() -> HV8 {
        HV8 { h: 0, v: 0 }
    }
}

const GAMMA8: [u16; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5,
    5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 14,
    14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 24, 24, 25, 25, 26, 27,
    27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37, 38, 39, 39, 40, 41, 42, 43, 44, 45, 46,
    47, 48, 49, 50, 50, 51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 72,
    73, 74, 75, 77, 78, 79, 81, 82, 83, 85, 86, 87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 104,
    105, 107, 109, 110, 112, 114, 115, 117, 119, 120, 122, 124, 126, 127, 129, 131, 133, 135, 137,
    138, 140, 142, 144, 146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 167, 169, 171, 173, 175,
    177, 180, 182, 184, 186, 189, 191, 193, 196, 198, 200, 203, 205, 208, 210, 213, 215, 218, 220,
    223, 225, 228, 231, 233, 236, 239, 241, 244, 247, 249, 252, 255,
];

impl Into<RGB8> for &HV8 {
    fn into(self) -> RGB8 {
        let RGB8 { r, g, b } = wheel(self.h);
        let v = self.v as usize;
        RGB8::new(
            (r as u16 * GAMMA8[v] / 255u16) as u8,
            (g as u16 * GAMMA8[v] / 255u16) as u8,
            (b as u16 * GAMMA8[v] / 255u16) as u8,
        )
        // let rgb = color::wheel(self.h);
        // brightness(iter, brightness)
    }
}
