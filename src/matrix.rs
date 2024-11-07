use smart_leds::RGB8;

use crate::{
    color::HV8,
    math::{self, Vec2},
};

pub const NUM_LEDS: usize = 291;
const MATRIX_MAP: [i16; 21 * 19] = [
    291, 291, 291, 291, 291, 291, 291, 291, 0, 1, 2, 3, 4, 5, 6, 7, 291, 291, 291, 291, 291, 291,
    291, 291, 291, 291, 16, 15, 14, 13, 12, 11, 10, 9, 8, 291, 291, 291, 291, 291, 291, 291, 291,
    291, 291, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 291, 291, 291, 291, 291, 291, 291, 291, 37,
    36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 291, 291, 291, 291, 291, 38, 39, 40, 41, 42, 43, 44,
    45, 46, 47, 48, 49, 50, 51, 52, 291, 291, 291, 68, 67, 66, 65, 64, 63, 62, 61, 60, 59, 58, 57,
    56, 55, 54, 53, 291, 291, 291, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84,
    85, 291, 102, 101, 100, 99, 98, 97, 96, 95, 94, 93, 92, 91, 90, 89, 88, 87, 86, 291, 291, 103,
    104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 291, 136, 135,
    134, 133, 132, 131, 130, 129, 128, 127, 126, 125, 124, 123, 122, 121, 120, 291, 291, 137, 138,
    139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 291, 291, 170, 169,
    168, 167, 166, 165, 164, 163, 162, 161, 160, 159, 158, 157, 156, 155, 154, 291, 291, 291, 171,
    172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 291, 291, 204,
    203, 202, 201, 200, 199, 198, 197, 196, 195, 194, 193, 192, 191, 190, 189, 188, 291, 291, 291,
    205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 291, 291,
    237, 236, 235, 234, 233, 232, 231, 230, 229, 228, 227, 226, 225, 224, 223, 222, 291, 291, 291,
    291, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 291, 291, 291,
    291, 291, 291, 291, 263, 262, 261, 260, 259, 258, 257, 256, 255, 254, 253, 291, 291, 291, 291,
    291, 291, 291, 291, 291, 264, 265, 266, 267, 268, 269, 270, 271, 272, 273, 291, 291, 291, 291,
    291, 291, 291, 291, 291, 282, 281, 280, 279, 278, 277, 276, 275, 274, 291, 291, 291, 291, 291,
    291, 291, 291, 291, 291, 291, 283, 284, 285, 286, 287, 288, 289, 290, 291, 291, 291,
];
pub const MATRIX_WIDTH: usize = 19;
pub const MATRIX_HEIGHT: usize = 21;

#[derive(Debug)]
pub enum Error {
    OutOfBounds,
}

pub fn set_matrix_oddr(v: math::Vec2, color: RGB8, data: &mut [RGB8; NUM_LEDS]) {
    match set_matrix((v.x + 10) as usize, (v.y + 10) as usize, color, data) {
        _ => (),
    }
}

pub fn set_matrix(
    x: usize,
    y: usize,
    color: RGB8,
    data: &mut [RGB8; NUM_LEDS],
) -> Result<i16, Error> {
    if x >= MATRIX_WIDTH || y >= MATRIX_HEIGHT {
        return Err(Error::OutOfBounds);
    }
    let addr = x + y * MATRIX_WIDTH;
    let led = MATRIX_MAP.get(addr).ok_or(Error::OutOfBounds)?;
    let rgb = data.get_mut(*led as usize).ok_or(Error::OutOfBounds)?;
    *rgb = color;
    Ok(*led)
}

pub fn get_matrix(x: usize, y: usize, data: &mut [RGB8; NUM_LEDS]) -> Result<(i16, RGB8), Error> {
    if x >= MATRIX_WIDTH || y >= MATRIX_HEIGHT {
        return Err(Error::OutOfBounds);
    }
    let addr = x + y * MATRIX_WIDTH;
    let led = MATRIX_MAP.get(addr).ok_or(Error::OutOfBounds)?;
    Ok((
        *led,
        data.get(*led as usize).cloned().ok_or(Error::OutOfBounds)?,
    ))
}
pub fn set_matrix_hv(
    x: usize,
    y: usize,
    hv: &HV8,
    data: &mut [HV8; NUM_LEDS],
) -> Result<i16, Error> {
    if x >= MATRIX_WIDTH || y >= MATRIX_HEIGHT {
        return Err(Error::OutOfBounds);
    }
    let addr = x + y * MATRIX_WIDTH;
    let led = MATRIX_MAP.get(addr).ok_or(Error::OutOfBounds)?;
    let out = data.get_mut(*led as usize).ok_or(Error::OutOfBounds)?;
    *out = *hv;
    Ok(*led)
}

pub fn led_addr(x: usize, y: usize) -> Result<usize, Error> {
    if x >= MATRIX_WIDTH || y >= MATRIX_HEIGHT {
        return Err(Error::OutOfBounds);
    }
    let addr = x + y * MATRIX_WIDTH;
    let led = MATRIX_MAP.get(addr).ok_or(Error::OutOfBounds)?;
    if *led >= 0 && (*led as usize) < NUM_LEDS {
        Ok(*led as usize)
    } else {
        Err(Error::OutOfBounds)
    }
}
pub fn adjacent(v: Vec2) -> [Vec2; 6] {
    let xshift = v.y.abs() % 2;
    let mut d = [
        Vec2::new(1, 0),
        Vec2::new(-1, 0),
        Vec2::new(-1 + xshift, 1),
        Vec2::new(0 + xshift, 1),
        Vec2::new(-1 + xshift, -1),
        Vec2::new(0 + xshift, -1),
    ];

    d.iter_mut().for_each(|f| *f = *f + v);
    d
}
