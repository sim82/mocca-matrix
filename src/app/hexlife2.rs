use bitset_core::BitSet;
use smart_leds::brightness;

use crate::{bitzet::Bitzet, math::Vec2, prelude::*, Error};

type BitzetN = Bitzet<128>;
// const LERP_TIME: i32 = 60 * 1;
// const PAUSE_TIME: i32 = 60 * 2;

const LERP_TIME: i32 = 60 * 5;
const PAUSE_TIME: i32 = 60 * 10;

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
        let RGB8 { r, g, b } = color::wheel(self.h);
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

pub struct Hexlife2 {
    black: BitzetN,

    i: usize,
    keep_on: [u32; NUM_LEDS / 32 + 1],

    // rainbow: Rainbow,
    rainbow: u8,

    last: [HV8; NUM_LEDS],

    next: [HV8; NUM_LEDS],
    f: i32,
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
    let led = crate::MATRIX_MAP.get(addr).ok_or(Error::OutOfBounds)?;
    let out = data.get_mut(*led as usize).ok_or(Error::OutOfBounds)?;
    *out = *hv;
    Ok(*led)
}

pub fn led_addr(x: usize, y: usize) -> Result<usize, Error> {
    if x >= MATRIX_WIDTH || y >= MATRIX_HEIGHT {
        return Err(Error::OutOfBounds);
    }
    let addr = x + y * MATRIX_WIDTH;
    let led = crate::MATRIX_MAP.get(addr).ok_or(Error::OutOfBounds)?;
    if *led >= 0 && (*led as usize) < crate::NUM_LEDS {
        Ok(*led as usize)
    } else {
        Err(Error::OutOfBounds)
    }
}

fn adjacent(v: Vec2) -> [Vec2; 6] {
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

pub fn new() -> Hexlife2 {
    let mut black = BitzetN::new();
    for (i, line) in input().iter().enumerate() {
        let mut c = line.chars();
        let mut x = 0i32;
        let mut y = 0i32;

        // data[i % NUM_LEDS] = RGB8::new(0, 255, 0);
        // let mut prev = None;
        fn reset_prev(prev: Option<(i16, RGB8)>, data: &mut [RGB8]) {
            if let Some((led, color)) = prev {
                data[led as usize] = color;
            }
        }
        loop {
            match c.next() {
                Some('e') => x += 1,
                Some('w') => x -= 1,
                Some('s') => match c.next() {
                    Some('e') => {
                        x += (y.abs() % 2);
                        y += 1
                    }
                    Some('w') => {
                        y += 1;
                        x -= (y.abs() % 2);
                    }
                    _ => break,
                },
                Some('n') => match c.next() {
                    Some('e') => {
                        x += (y.abs() % 2);
                        y -= 1
                    }
                    Some('w') => {
                        y -= 1;
                        x -= y.abs() % 2;
                    }
                    _ => break,
                },
                None => break,

                _ => break,
            }
        }

        if black.contains(&Vec2 { x, y }) {
            black.remove(&Vec2 { x, y });
        } else {
            black.insert(Vec2 { x, y });
        }
    }

    Hexlife2 {
        black,
        i: 0,
        keep_on: [0u32; NUM_LEDS / 32 + 1],
        rainbow: 0,
        next: [HV8::zero(); NUM_LEDS],
        last: [HV8::zero(); NUM_LEDS],
        f: LERP_TIME,
    }
}

impl app::App for Hexlife2 {
    fn tick(&mut self, led_data: &mut [RGB8; NUM_LEDS]) {
        // let mut rainbow = Rainbow::step(7);

        if self.f >= LERP_TIME + PAUSE_TIME {
            let mut black_new = Bitzet::new();

            for v in self.black.iter() {
                let n = adjacent(v)
                    .iter()
                    .filter(|v| self.black.contains(*v))
                    .count();
                if (1..=2).contains(&n) {
                    black_new.insert(v);
                }
            }

            let white = self
                .black
                .iter()
                .flat_map(|v| core::array::IntoIter::new(adjacent(v)))
                .collect::<BitzetN>();

            let white = white.difference(&self.black);
            for v in white.iter() {
                let n = adjacent(v)
                    .iter()
                    .filter(|v| self.black.contains(*v))
                    .count();
                if n == 2 && v.x.abs() < 15 && v.y.abs() < 15 {
                    black_new.insert(v);
                }
            }

            core::mem::swap(&mut self.black, &mut black_new);
            let black_old = black_new; // rebind to new name

            self.last = self.next;

            self.next.iter_mut().for_each(|hv| hv.v = 0);
            self.keep_on.fill(0);
            for v in self.black.iter() {
                if let Ok(addr) = led_addr((v.x + 10) as usize, (v.y + 10) as usize) {
                    self.next[addr].h = self.rainbow;
                    self.rainbow += 7;
                    if !black_old.contains(&v) {
                        self.last[addr].h = self.next[addr].h;
                    }
                    self.next[addr].v = 255;
                }
            }
            self.f = 0;

            for (out, hv) in led_data.iter_mut().zip(self.last.iter()) {
                *out = hv.into();
            }
            // *led_data = self.last;
        } else if self.f <= LERP_TIME {
            for (out, (last, next)) in led_data
                .iter_mut()
                .zip(self.last.iter().zip(self.next.iter()))
            {
                let h =
                    last.h as i32 + (next.h as i32 - last.h as i32) * (self.f as i32) / LERP_TIME;
                let v =
                    last.v as i32 + (next.v as i32 - last.v as i32) * (self.f as i32) / LERP_TIME;

                *out = (&HV8 {
                    h: h as u8,
                    v: v as u8,
                })
                    .into();
            }
        }
        self.i = self.i.overflowing_add(1).0;
        self.f = self.f.overflowing_add(1).0;
    }
}

fn input() -> &'static [&'static str] {
    &[
        "eeeee",
        "wwwwwwswsw",
        "neneeseswswswee",
        "w",
        "wwwwwwswswee",
        "wnwnwwswswsese",
        "wwwwwwnenwwsw",
        "wnwnwwswsw",
        "eeeeene",
        "eeeeese",
        "wnwnw",
        "wnwnww",
        "neneesesw",
        "wwwwwwnenww",
        "ne",
        "wnwnwwswswse",
        "wnwnww",
        "neneeseswswswe",
        "wwwwww",
        "eeeeesesw",
        "nene",
        "wwwwwwswswe",
        "neneeseswsw",
        "wwwwwwne",
        "eeeeenenw",
        "wnwnwwsw",
        "neneese",
        "wnwnwwswswsesee",
        "wnwnwwswswseseene",
        "wnw",
        "wwwwwwnenw",
        "wwwwwwsw",
        "nenee",
        "neneeseswswsw",
    ]
}
