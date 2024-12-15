use crate::{matrix, prelude::*};
use rand::{rngs::SmallRng, Rng, SeedableRng};

const TTL_MAX: usize = 60;

#[derive(Default, Copy, Clone)]
struct Seed {
    pos: Vec2,
    ttl: Option<usize>,
}
impl Seed {
    pub fn new(x: i32, y: i32) -> Self {
        Seed {
            pos: Vec2::new(x, y),
            ttl: Some(TTL_MAX),
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
    fn tick(&mut self, led_data: &mut [RGB8; NUM_LEDS]) {
        // self.data[10 * 19 + 10] = 1.0;
        // {
        //     let spawn = Vec2::new(
        //         self.rng.gen_range(matrix::MATRIX_X) as i32,
        //         self.rng.gen_range(17..matrix::MATRIX_HEIGHT) as i32,
        //     );
        //     let spawn_temp = self.rng.gen_range(0.2..0.6);
        //     self.set(spawn, spawn_temp);
        // }
        if self.rng.gen_bool(0.5) {
            let seed = &mut self.seeds[self.rng.gen_range(0..self.seeds.len())];
            if seed.ttl.is_none() {
                // seed.ttl = self.rng.gen_range((TTL_MAX / 2)..=TTL_MAX);
                seed.ttl = Some(TTL_MAX);
            }
        }
        for s in &mut self.seeds {
            if let Some(ttl) = &mut s.ttl {
                *ttl -= 1;
                if *ttl == 0 {
                    s.ttl = None;
                }
            }
        }
        for s in self.seeds {
            if let Some(ttl) = s.ttl {
                self.set(s.pos, 1.0 - (ttl as f32 / TTL_MAX as f32));
            }
        }
        let mut new_data = self.data.clone();
        let bias_range = 0.2;
        let bias = self.rng.gen_range(0.0..bias_range);
        for y in 0..21 {
            for x in 0..19 {
                let v = Vec2::new(x, y);
                let adj = matrix::adjacent(v);
                self.set(
                    v,
                    self.get(v) * 0.76
                        + self.get(adj[2]) * bias
                        + self.get(adj[3]) * (bias_range - bias),
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
