use crate::{bitzet::Bitzet, math::Vec2, prelude::*};

type BitzetN = Bitzet<128>;
// const LERP_TIME: i32 = 60 * 1;
// const PAUSE_TIME: i32 = 60 * 2;

const LERP_TIME: i32 = 60 * 5;
const PAUSE_TIME: i32 = 60 * 10;

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
                        x += y.abs() % 2;
                        y += 1
                    }
                    Some('w') => {
                        y += 1;
                        x -= y.abs() % 2;
                    }
                    _ => break,
                },
                Some('n') => match c.next() {
                    Some('e') => {
                        x += y.abs() % 2;
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
                    .filter(|v| self.black.contains(v))
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
                    .filter(|v| self.black.contains(v))
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
                    last.h as i32 + (next.h as i32 - last.h as i32) * self.f / LERP_TIME;
                let v =
                    last.v as i32 + (next.v as i32 - last.v as i32) * self.f / LERP_TIME;

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
