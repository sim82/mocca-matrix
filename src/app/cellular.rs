use crate::{matrix, prelude::*};
use defmt::info;
use rand::{rngs::SmallRng, Rng, SeedableRng};

const TTL_MAX: usize = 60;

#[derive(Default, Copy, Clone)]
struct Seed {
    pos: Vec2,
    fuel: f32,
    burning: bool,
    temperature: f32,
}
impl Seed {
    pub fn new(x: i32, y: i32) -> Self {
        Seed {
            pos: Vec2::new(x, y),
            fuel: 1.0,
            burning: false,
            temperature: 0.0,
        }
    }
}
pub struct Fire {
    data: [f32; 21 * 19],
    count: u8,
    rng: SmallRng,
    seeds: [Seed; 16],
    bias: f32,
}
pub fn new() -> Fire {
    let mut data = [0.0; MATRIX_HEIGHT * MATRIX_WIDTH];
    data[10 * 19 + 10] = 1.0;
    let seeds = [
        Seed::new(2, 15),
        Seed::new(3, 16),
        Seed::new(4, 16),
        Seed::new(5, 16),
        Seed::new(6, 16),
        Seed::new(6, 17),
        Seed::new(7, 18),
        Seed::new(7, 19),
        Seed::new(8, 20),
        Seed::new(9, 20),
        Seed::new(10, 20),
        Seed::new(11, 20),
        Seed::new(12, 20),
        Seed::new(13, 20),
        Seed::new(14, 20),
        Seed::new(15, 20),
    ];
    Fire {
        data,
        count: 0,
        rng: SmallRng::seed_from_u64(0),
        seeds,
        bias: 0.5,
    }
}

impl App for Fire {
    fn tick(&mut self, led_data: &mut [RGB8; NUM_LEDS], env: &Env) {
        // vary activity between ~ 40 - 100 db
        // FIXME: the second mems behaves weirly in the complete build. Maybe noise?
        let act = ((env.spl_db - 55.0) / 50.0).clamp(0.03, 1.0);
        // let act = ((env.spl_db - 45.0) / 60.0).clamp(0.03, 1.0);
        info!("act: {}", act);
        // if self.rng.gen_bool(act as f64) {
        //     let seed = &mut self.seeds[self.rng.gen_range(0..self.seeds.len())];
        //     if seed.ttl.is_none() {
        //         // seed.ttl = self.rng.gen_range((TTL_MAX / 2)..=TTL_MAX);
        //         seed.ttl = Some(TTL_MAX);
        //     }
        // }
        // for s in &mut self.seeds {
        //     if let Some(ttl) = &mut s.ttl {
        //         *ttl -= 1;
        //         if *ttl == 0 {
        //             s.ttl = None;
        //         }
        //     }
        // }
        // for s in self.seeds {
        //     if let Some(ttl) = s.ttl {
        //         self.set(s.pos, 1.0 - (ttl as f32 / TTL_MAX as f32));
        //     }
        // }
        for s in self.seeds {
            self.set(s.pos, s.temperature);
        }
        let mut burning = [false; 16];
        for (b, s) in burning.iter_mut().zip(self.seeds.iter()) {
            *b = s.burning;
        }
        for (i, s) in &mut self.seeds.iter_mut().enumerate() {
            if s.burning {
                s.fuel -= 0.032;
                if s.fuel <= 0.0 {
                    s.burning = false;
                }
                s.temperature = (s.temperature + 0.05).clamp(0.0, 1.0);
            } else {
                let mut r = 0.005;
                if i > 0 && burning[i - 1] {
                    r += 0.005;
                }
                if i < burning.len() - 1 && burning[i + 1] {
                    r += 0.005;
                }
                s.burning = self.rng.gen_bool(r);
            }
            s.temperature =
                0.00_f32.max(s.temperature - 0.02) + self.rng.gen_range(0.005f32..0.01f32);
            s.fuel = (s.fuel + (0.03 * act)).clamp(0.0f32, 1.0f32);
        }
        // let bias_range = 0.2;
        self.bias = (self.bias + self.rng.gen_range(-0.1..0.1) * 0.5).clamp(0.15, 0.85);
        // info!("bias: {}", self.bias);
        let mut new_data = self.data.clone();
        // let bias = self.rng.gen_range(0.0..bias_range);
        let feedback = 0.87;
        let up = 0.1;
        for y in 0..21 {
            for x in 0..19 {
                let v = Vec2::new(x, y);
                let adj = matrix::adjacent(v);
                self.set(
                    v,
                    self.get(v) * feedback
                        + self.get(adj[2]) * self.bias * up
                        + self.get(adj[3]) * (1.0 - self.bias) * up,
                )
            }
        }
        let r = 255.0;
        let g = 80.0;
        let b = 8.0;
        for (data, led) in self.data.iter().zip(crate::matrix::MATRIX_MAP.iter()) {
            let i = *led as usize;
            let data = data.clamp(0.0, 1.0);
            if i < NUM_LEDS {
                led_data[i] = RGB8::new(
                    ((r * data) as u8).clamp(0, 255),
                    ((g * data) as u8).clamp(0, 255),
                    ((b * data) as u8).clamp(0, 255),
                );
                // led_data[i] = led_data[i] = (&HV8 {
                //     h: 20,
                //     // v: self.count,
                //     v: (data.clamp(0.0, 1.0) * 255.0) as u8,
                // })
                //     .into();
            }
        }
        self.count = self.count.wrapping_add(1);
    }
}

impl Fire {
    fn set(&mut self, v: Vec2, f: f32) {
        if v.x < 0 || v.x as usize >= MATRIX_WIDTH || v.y < 0 || v.y as usize >= MATRIX_HEIGHT {
            return;
        }
        let addr = MATRIX_WIDTH * v.y as usize + v.x as usize;
        self.data[addr] = f;
    }
    fn get(&self, v: Vec2) -> f32 {
        if v.x < 0 || v.x as usize >= MATRIX_WIDTH || v.y < 0 || v.y as usize >= MATRIX_HEIGHT {
            return 0.0;
        }
        let addr = MATRIX_WIDTH * v.y as usize + v.x as usize;
        self.data[addr]
    }
    // fn get_mut(&mut self, v: Vec2) -> &mut f32 {
    //             if v.x < 0 || v.x as usize >= MATRIX_WIDTH || v.y < 0 || v.y as usize >= MATRIX_HEIGHT {
    //         return 0.0;
    //     }
    //     let addr = MATRIX_WIDTH * v.y as usize + v.x as usize;
    //     self.data[addr]
    // }
}

pub struct FireWorks {
    data: [f32; 21 * 19],
    count: u8,
    rng: SmallRng,
    // seeds: [Seed; 16],
}

impl FireWorks {
    pub fn new() -> FireWorks {
        let mut data = [0.0; MATRIX_HEIGHT * MATRIX_WIDTH];

        data[10 * 19 + 10] = 1.0;
        FireWorks {
            data,
            count: 0,
            rng: SmallRng::seed_from_u64(0),
            // seeds,
        }
    }
    fn set(&mut self, v: Vec2, f: f32) {
        if v.x < 0 || v.x as usize >= MATRIX_WIDTH || v.y < 0 || v.y as usize >= MATRIX_HEIGHT {
            return;
        }
        let addr = MATRIX_WIDTH * v.y as usize + v.x as usize;
        self.data[addr] = f;
    }
    fn get(&self, v: Vec2) -> f32 {
        if v.x < 0 || v.x as usize >= MATRIX_WIDTH || v.y < 0 || v.y as usize >= MATRIX_HEIGHT {
            return 0.0;
        }
        let addr = MATRIX_WIDTH * v.y as usize + v.x as usize;
        self.data[addr]
    }
}
impl App for FireWorks {
    fn tick(&mut self, led_data: &mut [RGB8; NUM_LEDS], _env: &Env) {
        let feedback = 0.31;
        let up = 0.1;
        for y in 0..21 {
            for x in 0..19 {
                let v = Vec2::new(x, y);
                let adj = matrix::adjacent(v);
                self.set(
                    v,
                    self.get(v) * feedback
                        + self.get(adj[0]) * up
                        + self.get(adj[1]) * up
                        + self.get(adj[2]) * up
                        + self.get(adj[3]) * up
                        + self.get(adj[4]) * up
                        + self.get(adj[5]) * up,
                )
            }
        }
        let r = 255.0;
        let g = 80.0;
        let b = 0.0;
        for (data, led) in self.data.iter().zip(crate::matrix::MATRIX_MAP.iter()) {
            let i = *led as usize;
            let data = data.clamp(0.0, 1.0);
            if i < NUM_LEDS {
                led_data[i] = RGB8::new(((r * data) as u8).clamp(0, 255), (g * data) as u8, 0);
                // led_data[i] = led_data[i] = (&HV8 {
                //     h: 20,
                //     // v: self.count,
                //     v: (data.clamp(0.0, 1.0) * 255.0) as u8,
                // })
                //     .into();
            }
        }
    }
}
