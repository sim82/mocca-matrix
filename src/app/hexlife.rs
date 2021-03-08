use bitset_core::BitSet;
use smart_leds::brightness;

use crate::{bitzet::Bitzet, math::Vec2, prelude::*};

type BitzetN = Bitzet<128>;
const LERP_TIME: i32 = 60 * 5;
const PAUSE_TIME: i32 = 60 * 5;

pub struct Hexlife {
    black: BitzetN,

    i: usize,
    keep_on: [u32; NUM_LEDS / 32 + 1],

    rainbow: Rainbow,

    last: [RGB8; NUM_LEDS],
    next: [RGB8; NUM_LEDS],
    f: i32,
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

impl app::App for Hexlife {
    fn new() -> Self {
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

                // reset_prev(prev, &mut data);
                // prev = get_matrix((x + 10) as usize, (y + 10) as usize, &mut data).ok();
                // set_matrix(
                //     (x + 10) as usize,
                //     (y + 10) as usize,
                //     RGB8::new(0, 255, 0),
                //     &mut data,
                // );
                // ws.write(brightness(data.iter().cloned(), 32)).unwrap();
                // delay.delay_ms(8u8);
            }
            // reset_prev(prev, &mut data);

            if black.contains(&Vec2 { x, y }) {
                black.remove(&Vec2 { x, y });
            } else {
                black.insert(Vec2 { x, y });
            }
            // set_matrix(
            //     (x + 10) as usize,
            //     (y + 10) as usize,
            //     RGB8::new(0, 0, 255),
            //     &mut data,
            // );
            // ws.write(brightness(data.iter().cloned(), 32)).unwrap();
        }

        Hexlife {
            black,
            i: 0,
            keep_on: [0u32; NUM_LEDS / 32 + 1],
            rainbow: Rainbow::step(7),
            next: [color::BLACK; NUM_LEDS],
            last: [color::BLACK; NUM_LEDS],
            f: LERP_TIME,
        }
    }

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

            self.black = black_new;

            self.last = self.next;
            self.next.fill(color::BLACK);
            self.keep_on.fill(0);
            for v in self.black.iter() {
                if let Ok(addr) = set_matrix(
                    (v.x + 10) as usize,
                    (v.y + 10) as usize,
                    self.rainbow.next().unwrap(),
                    &mut self.next,
                ) {
                    self.keep_on.bit_set(addr as usize);
                }
            }
            self.f = 0;

            *led_data = self.last;
        } else if self.f <= LERP_TIME {
            // for i in 0..NUM_LEDS {
            //     if !self.keep_on.bit_test(i) {
            //         let v = &mut led_data[i];
            //         // let old = [v.clone(); 1];
            //         *v = brightness(core::iter::once(*v), 222).next().unwrap();
            //     }
            // }

            for (out, (last, next)) in led_data
                .iter_mut()
                .zip(self.last.iter().zip(self.next.iter()))
            {
                let r =
                    last.r as i32 + (next.r as i32 - last.r as i32) * (self.f as i32) / LERP_TIME;
                let g =
                    last.g as i32 + (next.g as i32 - last.g as i32) * (self.f as i32) / LERP_TIME;
                let b =
                    last.b as i32 + (next.b as i32 - last.b as i32) * (self.f as i32) / LERP_TIME;

                out.r = r as u8;
                out.g = g as u8;
                out.b = b as u8;
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
