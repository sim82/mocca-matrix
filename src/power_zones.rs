use super::NUM_LEDS;

const START0: usize = 0;
const SIZE0: usize = 8 + 9 + 10 + 11 + 15 + 16 + 17;
const START1: usize = SIZE0;
const SIZE1: usize = 17 + 17 + 17 + 17;
const START2: usize = START1 + SIZE1;
const SIZE2: usize = 17 + 17 + 17 + 17;
const START3: usize = START2 + SIZE2;
const SIZE3: usize = 16 + 15 + 11 + 10 + 9 + 8;
const END3: usize = START3 + SIZE3;
pub const NUM_ZONES: usize = 4;
const ZONES: [core::ops::Range<usize>; NUM_ZONES] =
    [START0..START1, START1..START2, START2..START3, START3..END3];

fn rgb8_to_power(c: &smart_leds::RGB8) -> u32 {
    let tmp = 122 * c.r as u32 + 121 * c.g as u32 + 121 * c.b as u32;
    tmp / 2550
}

fn estimate_current(data: &[smart_leds::RGB8]) -> u32 {
    data.iter().map(|c| rgb8_to_power(c)).sum::<u32>()
}

pub fn estimate_current_all(data: &[smart_leds::RGB8; NUM_LEDS]) -> [u32; NUM_ZONES] {
    let mut out = [0; NUM_ZONES];
    for (i, range) in ZONES.iter().cloned().enumerate() {
        out[i] = 78 + estimate_current(&data[range]);
    }
    out
}

pub fn limit_current(
    data: &mut [smart_leds::RGB8; NUM_LEDS],
    limit: &[u32; NUM_ZONES],
) -> [Option<u32>; NUM_ZONES] {
    // const LIMIT: u32 = 1100;
    let mut ret = [None; NUM_ZONES];
    for (i, range) in ZONES.iter().cloned().enumerate() {
        let data = &mut data[range];
        let current = 78 + estimate_current(data);

        if current <= limit[i] {
            continue;
        }
        const MUL: u32 = 1000;
        let f = limit[i] * MUL / current;
        // let f = LIMIT as f32 / current as f32;

        data.iter_mut().for_each(|v| {
            v.r = ((v.r as u32) * f / MUL) as u8;
            v.g = ((v.g as u32) * f / MUL) as u8;
            v.b = ((v.b as u32) * f / MUL) as u8;
            // v.r = ((v.r as f32) * f) as u8;
            // v.g = ((v.g as f32) * f) as u8;
            // v.b = ((v.b as f32) * f) as u8;
        });
        ret[i] = Some(f);
    }
    ret
}

const CURRENT_MAX: u32 = 1100;
const CURRENT_RATED: u32 = 500;
const NUM_MEASUREMENTS: usize = 60;
#[derive(Copy, Clone)]
pub struct DynamicLimit {
    measurements: [u32; NUM_MEASUREMENTS],
    i: usize,

    acc: u32,
    acc_count: u32,
    pub limit: u32,
}

impl Default for DynamicLimit {
    fn default() -> Self {
        Self {
            measurements: [0; NUM_MEASUREMENTS],
            i: 0,
            limit: CURRENT_MAX,

            acc: 0,
            acc_count: 0,
        }
    }
}

impl DynamicLimit {
    pub fn commit(&mut self) {
        let current = if self.acc_count != 0 {
            self.acc / self.acc_count
        } else {
            0
        };
        self.acc = 0;
        self.acc_count = 0;

        if self.i >= NUM_MEASUREMENTS {
            self.i %= NUM_MEASUREMENTS;
        }
        self.measurements[self.i] = current;
        self.i += 1;

        let energy = self.measurements.iter().sum::<u32>();
        let budget = NUM_MEASUREMENTS as u32 * CURRENT_RATED;

        if energy > budget {
            let f = energy as f32 / budget as f32;
            let f = if f < 1.0 {
                1.0
            } else if f > 2.0 {
                2.0
            } else {
                f
            };
            self.limit = CURRENT_RATED + ((CURRENT_MAX - CURRENT_RATED) as f32 * (2.0 - f)) as u32;
        }
    }
    pub fn add_measurement(&mut self, current: u32) {
        self.acc += current;
        self.acc_count += 1;
    }

    pub fn get_limit(&self) -> u32 {
        self.limit
    }
}
