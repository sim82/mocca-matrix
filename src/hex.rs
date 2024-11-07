use crate::math::Vec2;
use core::ops;
use num_traits::{self, float::FloatCore, Num};

// mostly based on https://www.redblobgames.com/grids/hexagons/

#[derive(Default, Debug, Clone, Copy)]
pub struct Cube {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Cube {
    pub fn new(x: i32, y: i32, z: i32) -> Cube {
        Cube { x, y, z }
    }
    pub fn zero() -> Cube {
        Cube::default()
    }
}

impl From<&Cube> for Cube {
    fn from(c: &Cube) -> Self {
        *c
    }
}

impl ops::Add for Cube {
    type Output = Cube;
    fn add(self, rhs: Self) -> Self::Output {
        Cube::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl ops::Sub for Cube {
    type Output = Cube;
    fn sub(self, rhs: Self) -> Self::Output {
        Cube::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl ops::AddAssign for Cube {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::Mul<i32> for Cube {
    type Output = Cube;

    fn mul(self, rhs: i32) -> Self::Output {
        Cube::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl ops::MulAssign<i32> for Cube {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl From<(i32, i32, i32)> for Cube {
    fn from(v: (i32, i32, i32)) -> Self {
        Cube::new(v.0, v.1, v.2)
    }
}

pub struct Hex {
    pub q: i32,
    pub r: i32,
}

impl From<Cube> for Hex {
    fn from(c: Cube) -> Self {
        Hex { q: c.x, r: c.z }
    }
}

impl From<Hex> for Cube {
    fn from(h: Hex) -> Self {
        Cube {
            x: h.q,
            y: -h.q - h.r,
            z: h.r,
        }
    }
}

// impl Into<Vec2> for Cube {
//     fn into(self) -> Vec2 {
//         let x = self.x + (self.z - (self.z & 1)) / 2;
//         let y = self.z;
//         Vec2 { x, y }
//     }
// }

impl From<Vec2> for Cube {
    fn from(v: Vec2) -> Cube {
        let x = v.x - (v.y - (v.y & 1)) / 2;
        let z = v.y;
        let y = -x - z;
        Cube { x, y, z }
    }
}

pub const CUBE_DIRECTIONS: [Cube; 6] = [
    Cube { x: 1, y: -1, z: 0 },
    Cube { x: 1, y: 0, z: -1 },
    Cube { x: 0, y: 1, z: -1 },
    Cube { x: -1, y: 1, z: 0 },
    Cube { x: -1, y: 0, z: 1 },
    Cube { x: 0, y: -1, z: 1 },
];

fn lerp<T: Num + Copy>(a: T, b: T, t: T) -> T {
    a + (b - a) * t
}

fn cube_lerp(a: &Cube, b: &Cube, t: f32) -> Cube {
    Cube {
        x: lerp(a.x as f32, b.x as f32, t) as i32,
        y: lerp(a.y as f32, b.y as f32, t) as i32,
        z: lerp(a.z as f32, b.z as f32, t) as i32,
    }
}

fn cube_round(x: f32, y: f32, z: f32) -> Cube {
    let mut rx = x.round();
    let mut ry = y.round();
    let mut rz = z.round();

    let x_diff = (rx - x).abs();
    let y_diff = (ry - y).abs();
    let z_diff = (rz - z).abs();

    if x_diff > y_diff && x_diff > z_diff {
        rx = -ry - rz
    } else if y_diff > z_diff {
        ry = -rx - rz
    } else {
        rz = -rx - ry
    }

    Cube {
        x: rx as i32,
        y: ry as i32,
        z: rz as i32,
    }
}

fn cube_distance(a: &Cube, b: &Cube) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs() / 2
}

pub fn cube_linedraw(a: &Cube, b: &Cube) -> (i32, [Cube; 20]) {
    let n = cube_distance(a, b);
    let mut res = [Cube::default(); 20];
    for i in 0..n.max(20) {
        let mut c = cube_lerp(a, b, 1f32 / n as f32 * i as f32);
        c.y = -c.x - c.z;
        res[i as usize] = c;
    }
    (n.max(20), res)
}

pub struct CubeLinedraw {
    a: Cube,
    b: Cube,
    n: i32,
    i: i32,
}

impl CubeLinedraw {
    pub fn new(a: Cube, b: Cube) -> Self {
        let n = cube_distance(&a, &b);
        CubeLinedraw { a, b, n, i: 0 }
    }
}

impl Iterator for CubeLinedraw {
    type Item = Cube;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.n {
            None
        } else {
            let t = 1f32 / self.n as f32 * self.i as f32;
            let x = lerp(self.a.x as f32, self.b.x as f32, t);
            let y = lerp(self.a.y as f32, self.b.y as f32, t);
            let z = lerp(self.a.z as f32, self.b.z as f32, t);
            self.i += 1;
            Some(cube_round(x, y, z))
        }
    }
}

pub mod prelude {
    pub use super::{Cube, Hex};
}
