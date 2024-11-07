use core::iter::FromIterator;

use crate::math::Vec2;
use bitset_core::BitSet;
//
// bit-set indexed by Vec2 type. Basically meant as a no_std drop in replacement for
// HashSet<Vec2> that I normally use to model a 2d-indexed bitmaps with unclear min/max
// bounds and shape (e.g in typial AoC puzzles)
// Uses the z-order curve (hence the stupid name) to map 2d coords onto a linear range (or four ranges, one per quadrant).
// Might seem like overkill compared to a 'simple' 2d array, but 2d arrays are not that nice without
// dynamic allocations after all (+ I hate those things).
//
// Nice bonus: points can be iterated in a continuous curve which is nice for visualizations (and for caches...)

// const N: usize = 128;
#[derive(Clone)]
pub struct Bitzet<const N: usize> {
    pub quadrants: [[u32; N]; 4],
    pub max: [usize; 4],
}

pub struct ZOrderIterator<'a, const N: usize> {
    z: usize,
    quadrant: usize,
    s: &'a Bitzet<N>,
}

impl<'a, const N: usize> Iterator for ZOrderIterator<'a, N> {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.quadrant >= 4 {
                return None;
            }
            if self.z > self.s.max[self.quadrant] {
                self.quadrant += 1;
                self.z = 0;
                continue;
            }
            if self.s.quadrants[self.quadrant].bit_test(self.z) {
                break;
            }
            self.z += 1;
        }
        let ret = zorder_inverse(self.z as u32);
        self.z += 1;
        match self.quadrant {
            0 => Some(Vec2::new(ret.x, ret.y)),
            1 => Some(Vec2::new(-ret.x, ret.y)),
            2 => Some(Vec2::new(ret.x, -ret.y)),
            3 => Some(Vec2::new(-ret.x, -ret.y)),
            _ => None,
        }
    }
}

impl<const N: usize> FromIterator<Vec2> for Bitzet<N> {
    fn from_iter<T: IntoIterator<Item = Vec2>>(iter: T) -> Self {
        let mut bz = Bitzet::new();
        iter.into_iter().for_each(|v| bz.insert(v));
        bz
    }
}
impl<const N: usize> Default for Bitzet<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Bitzet<N> {
    pub fn new() -> Bitzet<N> {
        Bitzet {
            quadrants: [[0; N]; 4],
            max: [0; 4],
        }
    }
    pub fn insert(&mut self, v: Vec2) {
        let z = zorder_abs(&v);
        let q = quadrant_index(&v);
        self.quadrants[q].bit_set(z);
        self.max[q] = self.max[q].max(z);
    }
    pub fn remove(&mut self, v: &Vec2) {
        self.quadrants[quadrant_index(v)].bit_reset(zorder_abs(v));
    }
    pub fn get(&self, v: &Vec2) -> bool {
        self.quadrants[quadrant_index(v)].bit_test(zorder_abs(v))
    }
    pub fn contains(&self, v: &Vec2) -> bool {
        // println!("contains: {:?}", v);
        self.get(v)
    }
    pub fn len(&self) -> usize {
        self.quadrants
            .iter()
            .zip(self.max.iter())
            .map(|(q, max)| q[0..=(max / 32)].bit_count())
            .sum::<usize>()

        // self.quadrants.iter().map(|q| q.bit_count()).sum::<usize>()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn difference(&self, other: &Self) -> Self {
        let mut q0 = self.quadrants[0];
        q0.bit_andnot(&other.quadrants[0]);
        let mut q1 = self.quadrants[1];
        q1.bit_andnot(&other.quadrants[1]);
        let mut q2 = self.quadrants[2];
        q2.bit_andnot(&other.quadrants[2]);
        let mut q3 = self.quadrants[3];
        q3.bit_andnot(&other.quadrants[3]);

        Bitzet {
            quadrants: [q0, q1, q2, q3],
            max: self.max,
        }
    }
    pub fn iter(&self) -> ZOrderIterator<'_, N> {
        ZOrderIterator {
            s: self,
            quadrant: 0,
            z: 0,
        }
    }
}
fn quadrant_index(v: &Vec2) -> usize {
    let xneg = if v.x < 0 { 1 } else { 0 };
    let yneg = if v.y < 0 { 1 } else { 0 };
    yneg * 2 + xneg
}
fn zorder_abs(v: &Vec2) -> usize {
    zorder(v.x.unsigned_abs(), v.y.unsigned_abs()) as usize
}
fn zorder(mut x: u32, mut y: u32) -> u32 {
    // from https://graphics.stanford.edu/~seander/bithacks.html

    const B: [u32; 4] = [0x55555555, 0x33333333, 0x0F0F0F0F, 0x00FF00FF];
    const S: [u32; 4] = [1, 2, 4, 8];

    // Interleave lower 16 bits of x and y, so the bits of x
    // are in the even positions and bits from y in the odd;
    // z gets the resulting 32-bit Morton Number.
    // x and y must initially be less than 65536.

    x = (x | (x << S[3])) & B[3];
    x = (x | (x << S[2])) & B[2];
    x = (x | (x << S[1])) & B[1];
    x = (x | (x << S[0])) & B[0];

    y = (y | (y << S[3])) & B[3];
    y = (y | (y << S[2])) & B[2];
    y = (y | (y << S[1])) & B[1];
    y = (y | (y << S[0])) & B[0];

    x | (y << 1)
}
fn zorder_inverse(z: u32) -> Vec2 {
    let mut x = z & 0x55555555;
    x = (x | (x >> 1)) & 0x33333333;
    x = (x | (x >> 2)) & 0x0F0F0F0F;
    x = (x | (x >> 4)) & 0x00FF00FF;
    x = (x | (x >> 8)) & 0x0000FFFF;

    let mut y = (z >> 1) & 0x55555555;
    y = (y | (y >> 1)) & 0x33333333;
    y = (y | (y >> 2)) & 0x0F0F0F0F;
    y = (y | (y >> 4)) & 0x00FF00FF;
    y = (y | (y >> 8)) & 0x0000FFFF;

    Vec2::new(x as i32, y as i32)
}

// TODO: check how to run tests on host
// #[cfg(test)]
// mod test {
//     use crate::bitzet::Bitzet;
//     use crate::math::Vec2;
//     #[test]
//     fn test_iter_basic() {
//         let mut bs = Bitzet::new();
//         bs.insert(Vec2::new(1, 1));
//         bs.insert(Vec2::new(2, 1));
//         bs.insert(Vec2::new(3, 1));

//         let mut bs2 = Bitzet::new();
//         bs2.insert(Vec2(2, 1));
//         let bs3 = bs.difference(&bs2);
//         let s = bs3.iter().collect::<Vec<_>>();
//         println!("s: {:?}", s);
//     }

//     #[test]
//     fn test_iter_4q() {
//         let mut bs = Bitzet::new();
//         bs.insert(Vec2::new(1, 1));
//         bs.insert(Vec2::new(2, 1));
//         bs.insert(Vec2::new(3, 1));

//         bs.insert(Vec2::new(-5, 2));
//         bs.insert(Vec2::new(3, -7));
//         bs.insert(Vec2::new(-12, -13));

//         bs.insert(Vec2::new(-5, 1));
//         bs.insert(Vec2::new(2, -7));
//         bs.insert(Vec2::new(-11, -13));

//         let mut bs2 = Bitzet::new();
//         bs2.insert(Vec2::new(2, 1));
//         bs2.insert(Vec2::new(-5, 2));
//         bs2.insert(Vec2::new(3, -7));
//         bs2.insert(Vec2::new(-12, -13));
//         let bs3 = bs.difference(&bs2);
//         let s = bs3.iter().collect::<Vec<_>>();
//         println!("s: {:?}", s);
//     }

//     #[test]
//     fn test_len() {
//         println!("z: {}", zorder(255, 255));
//         let bz = [
//             Vec2::new(1, 1),
//             Vec2::new(255, 255),
//             Vec2::new(-1, 1),
//             Vec2::new(-255, 255),
//             Vec2::new(1, -1),
//             Vec2::new(255, -255),
//             Vec2::new(-1, -1),
//             Vec2::new(-255, -255),
//         ]
//         .iter()
//         .cloned()
//         .collect::<Bitzet>();
//         assert_eq!(bz.len(), 8);
//     }
//     #[test]
//     fn zorder3_test() {
//         assert_eq!(zorder(0, 0), 0);
//         assert_eq!(zorder(3, 5), 0b100111);
//         assert_eq!(zorder(6, 2), 0b011100);
//         assert_eq!(zorder(7, 7), 0b111111);
//     }

//     #[test]
//     fn test_zinv2() {
//         assert_eq!(zorder_inverse(0b0), Vec2(0, 0));
//         assert_eq!(zorder_inverse(0b100110), Vec2(0b10, 0b101));
//         assert_eq!(zorder_inverse(0b111101), Vec2(0b111, 0b110));

//         assert_eq!(
//             zorder_inverse(0b01010101010101010101010101010101),
//             Vec2(0b1111111111111111, 0b0)
//         );

//         assert_eq!(
//             zorder_inverse(0b10101010101010101010101010101010),
//             Vec2(0b0, 0b1111111111111111)
//         );
//         let v = zorder_inverse(0b111100);
//         println!("{:b} {:b}", v.x(), v.y());
//     }

//     #[test]
//     fn test1() {
//         let mut x = 0u32;
//         for i in 0..10 {
//             println!("x: {:b}", x);

//             x = (x + 0xaaaaaaab) & 0x55555555;
//         }
//     }
// }
