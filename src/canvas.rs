use smart_leds::RGB8;

use crate::{color, set_matrix_oddr, NUM_LEDS};

use super::hex;

pub trait Canvas {
    fn clear(&mut self);
    fn line(&mut self, a: hex::Cube, b: hex::Cube, color: RGB8);
    // fn apply(&mut self);
    // fn data(&mut self) -> &mut [RGB8; NUM_LEDS];
}

impl Canvas for [RGB8; NUM_LEDS] {
    fn line(&mut self, a: hex::Cube, b: hex::Cube, color: RGB8) {
        for c in hex::CubeLinedraw::new(a.into(), b.into()) {
            set_matrix_oddr(c.into(), color, self);
        }
    }

    fn clear(&mut self) {
        self.fill(color::BLACK);
    }
}
