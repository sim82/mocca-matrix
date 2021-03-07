use bitset_core::BitSet;
use smart_leds::brightness;

use crate::{bitzet::Bitzet, math::Vec2, prelude::*};

type BitzetN = Bitzet<128>;

pub struct Hexlife {
    black: BitzetN,

    i: usize,
    keep_on: [u32; NUM_LEDS / 32 + 1],

    rainbow: Rainbow,
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
        }
    }

    fn tick(&mut self, led_data: &mut [RGB8; NUM_LEDS]) {
        // {
        //     let mut rainbow = Rainbow::step(3);
        //     for _ in 0..100 {
        //         let c = rainbow.next().unwrap();
        //         data.iter_mut().for_each(|v| {
        //             if v.r != 0 || v.g != 0 || v.b != 0 {
        //                 *v = c
        //             }
        //         });
        //         ws.write(brightness(data.iter().cloned(), 32)).unwrap();
        //     }
        // }
        // while button.is_high().unwrap() {}

        // let mut rainbow = Rainbow::step(7);
        let warp_mode = false; //button.is_low().unwrap();
        let hold_mode = false; //button.is_low().unwrap();
        if self.i % 60 == 0 || warp_mode {
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

            if !hold_mode {
                self.black = black_new;

                // {
                //     let mut num_buffer = [0u8; 20];
                //     let mut text = ArrayString::<[_; 100]>::new();
                //     text.push_str("num: ");
                //     text.push_str(black.len().numtoa_str(10, &mut num_buffer));
                //     console.write(&text, Some(0));
                // }
            }
            self.keep_on.fill(0);
            if warp_mode {
                led_data.fill(RGB8::default());
            }
            for v in self.black.iter() {
                if let Ok(addr) = set_matrix(
                    (v.x + 10) as usize,
                    (v.y + 10) as usize,
                    self.rainbow.next().unwrap(),
                    led_data,
                ) {
                    self.keep_on.bit_set(addr as usize);
                }
            }
        }
        self.i = self.i.overflowing_add(1).0;

        if !warp_mode {
            for i in 0..NUM_LEDS {
                if !self.keep_on.bit_test(i) {
                    let v = &mut led_data[i];
                    // let old = [v.clone(); 1];
                    *v = brightness(core::iter::once(*v), 222).next().unwrap();
                }
            }
        }
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
