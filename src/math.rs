use core::ops;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

impl Vec2 {
    pub fn new(x: i32, y: i32) -> Vec2 {
        Vec2 { x, y }
    }
    pub fn manhattan(&self) -> i32 {
        self.x.abs() + self.y.abs()
    }
    pub fn rotate_right90(&self) -> Vec2 {
        // matrix:
        // x      0       -1
        // y      1        0
        Vec2::new(self.y, -self.x)
    }
    pub fn rotate_left90(&self) -> Vec2 {
        // matrix:
        // x       0       1
        // y      -1       0
        Vec2::new(-self.y, self.x)
    }
}

impl ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl ops::Mul<i32> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: i32) -> Self::Output {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl ops::MulAssign<i32> for Vec2 {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl From<(i32, i32)> for Vec2 {
    fn from(v: (i32, i32)) -> Self {
        Vec2::new(v.0, v.1)
    }
}
impl From<char> for Vec2 {
    fn from(c: char) -> Self {
        match c {
            'N' => Vec2::new(0, 1),
            'S' => Vec2::new(0, -1),
            'E' => Vec2::new(1, 0),
            'W' => Vec2::new(-1, 0),
            _ => panic!("unhandled direction: {}", c),
        }
    }
}
impl From<super::hex::Cube> for Vec2 {
    fn from(c: super::hex::Cube) -> Self {
        let x = c.x + (c.z - (c.z & 1)) / 2;
        let y = c.z;
        Vec2 { x, y }
    }
}

impl From<&Vec2> for Vec2 {
    fn from(v: &Vec2) -> Self {
        v.clone()
    }
}

// #[cfg(test)]
// mod test_vec2 {
//     use super::*;
//     #[test]
//     fn test_add() {
//         assert_eq!(Vec2(0, 0) + Vec2(2, 3), Vec2(2, 3));
//         assert_eq!(Vec2(4, 5) + Vec2(2, 3), Vec2(6, 8));
//         assert_eq!(Vec2(2, 3) + Vec2(-2, -3), Vec2(0, 0));
//     }
//     #[test]
//     fn test_addassign() {
//         let mut v = Vec2(0, 0);
//         v += Vec2(2, 3);
//         assert_eq!(v, Vec2(2, 3));
//         v += Vec2(-2, -3);
//         assert_eq!(v, Vec2(2, 3) + Vec2(-2, -3));
//     }
//     #[test]
//     fn test_mul() {
//         assert_eq!(Vec2(2, 3) * 8, Vec2(2 * 8, 3 * 8));
//         assert_eq!(Vec2(-2, -3) * 8, Vec2(-2 * 8, -3 * 8));
//         assert_eq!(Vec2(4, 5) * 0, Vec2(0, 0));
//     }
//     #[test]
//     fn test_mulassign() {
//         let mut v = Vec2(2, 3);
//         v *= 27;
//         assert_eq!(v, Vec2(2, 3) * 27);
//         v *= -3;
//         assert_eq!(v, Vec2(2, 3) * 27 * -3);
//         v *= 0;
//         assert_eq!(v, Vec2(0, 0));
//     }
//     #[test]
//     fn test_from() {
//         let v: Vec2 = (7, -8).into();
//         assert_eq!(v, Vec2(7, -8));

//         let n: Vec2 = 'N'.into();
//         let s: Vec2 = 'S'.into();
//         let e: Vec2 = 'E'.into();
//         let w: Vec2 = 'W'.into();
//         assert_eq!(n, s * -1);
//         assert_eq!(e, w * -1);
//         assert_ne!(n, e);
//         assert_ne!(n, Vec2(0, 0));
//     }
// }
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Vec3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl Vec3 {
    pub fn new(x: i32, y: i32, z: i32) -> Vec3 {
        Vec3 { x, y, z }
    }
    pub fn manhattan(&self) -> i32 {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
}

impl ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::Mul<i32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: i32) -> Self::Output {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl ops::MulAssign<i32> for Vec3 {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl From<(i32, i32, i32)> for Vec3 {
    fn from(v: (i32, i32, i32)) -> Self {
        Vec3::new(v.0, v.1, v.2)
    }
}
// #[cfg(test)]
// mod test_vec3 {
//     use super::*;
//     #[test]
//     fn test_add() {
//         assert_eq!(Vec3(0, 0, 0) + Vec3(2, 3, 1), Vec3(2, 3, 1));
//         assert_eq!(Vec3(4, 5, 6) + Vec3(2, 3, 4), Vec3(6, 8, 10));
//         assert_eq!(Vec3(2, 3, 4) + Vec3(-2, -3, -4), Vec3(0, 0, 0));
//     }
//     #[test]
//     fn test_addassign() {
//         let mut v = Vec3(0, 0, 0);
//         v += Vec3(2, 3, 4);
//         assert_eq!(v, Vec3(2, 3, 4));
//         v += Vec3(-2, -3, -4);
//         assert_eq!(v, Vec3(2, 3, 4) + Vec3(-2, -3, -4));
//     }
//     #[test]
//     fn test_mul() {
//         assert_eq!(Vec3(2, 3, 4) * 8, Vec3(2 * 8, 3 * 8, 4 * 8));
//         assert_eq!(Vec3(-2, -3, -4) * 8, Vec3(-2 * 8, -3 * 8, -4 * 8));
//         assert_eq!(Vec3(4, 5, 6) * 0, Vec3(0, 0, 0));
//     }
//     #[test]
//     fn test_mulassign() {
//         let mut v = Vec3(2, 3, 4);
//         v *= 27;
//         assert_eq!(v, Vec3(2, 3, 4) * 27);
//         v *= -3;
//         assert_eq!(v, Vec3(2, 3, 4) * 27 * -3);
//         v *= 0;
//         assert_eq!(v, Vec3(0, 0, 0));
//     }
//     #[test]
//     fn test_from() {
//         let v: Vec3 = (7, -8, 11).into();
//         assert_eq!(v, Vec3(7, -8, 11));
//     }
// }

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Vec4 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub w: i32,
}
impl Vec4 {
    pub fn new(x: i32, y: i32, z: i32, w: i32) -> Vec4 {
        Vec4 { x, y, z, w }
    }

    pub fn manhattan(&self) -> i32 {
        self.x.abs() + self.y.abs() + self.z.abs() + self.w.abs()
    }
}

impl ops::Add for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: Self) -> Self::Output {
        Vec4::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

impl ops::AddAssign for Vec4 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }
}

impl ops::Mul<i32> for Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: i32) -> Self::Output {
        Vec4::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

impl ops::MulAssign<i32> for Vec4 {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self.w *= rhs;
    }
}

impl From<(i32, i32, i32, i32)> for Vec4 {
    fn from(v: (i32, i32, i32, i32)) -> Self {
        Vec4::new(v.0, v.1, v.2, v.3)
    }
}
// #[cfg(test)]
// mod test_vec4 {
//     use super::*;
//     #[test]
//     fn test_add() {
//         assert_eq!(Vec4(0, 0, 0, 0) + Vec4(2, 3, 1, 4), Vec4(2, 3, 1, 4));
//         assert_eq!(Vec4(4, 5, 6, 7) + Vec4(2, 3, 4, 5), Vec4(6, 8, 10, 12));
//         assert_eq!(Vec4(2, 3, 4, 5) + Vec4(-2, -3, -4, -5), Vec4(0, 0, 0, 0));
//     }
//     #[test]
//     fn test_addassign() {
//         let mut v = Vec4(0, 0, 0, 0);
//         v += Vec4(2, 3, 4, 5);
//         assert_eq!(v, Vec4(2, 3, 4, 5));
//         v += Vec4(-2, -3, -4, -5);
//         assert_eq!(v, Vec4(2, 3, 4, 5) + Vec4(-2, -3, -4, -5));
//     }
//     #[test]
//     fn test_mul() {
//         assert_eq!(Vec4(2, 3, 4, 5) * 8, Vec4(2 * 8, 3 * 8, 4 * 8, 5 * 8));
//         assert_eq!(
//             Vec4(-2, -3, -4, -5) * 8,
//             Vec4(-2 * 8, -3 * 8, -4 * 8, -5 * 8)
//         );
//         assert_eq!(Vec4(4, 5, 6, 7) * 0, Vec4(0, 0, 0, 0));
//     }
//     #[test]
//     fn test_mulassign() {
//         let mut v = Vec4(2, 3, 4, 5);
//         v *= 27;
//         assert_eq!(v, Vec4(2, 3, 4, 5) * 27);
//         v *= -3;
//         assert_eq!(v, Vec4(2, 3, 4, 5) * 27 * -3);
//         v *= 0;
//         assert_eq!(v, Vec4(0, 0, 0, 0));
//     }
//     #[test]
//     fn test_from() {
//         let v: Vec4 = (7, -8, 11, -17).into();
//         assert_eq!(v, Vec4(7, -8, 11, -17));
//     }
// }
