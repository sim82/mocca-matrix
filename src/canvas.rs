use smart_leds::RGB8;

use crate::color;

use super::hex;
use crate::prelude::*;

pub trait Canvas {
    fn clear(&mut self);
    fn line(&mut self, a: hex::Cube, b: hex::Cube, color: RGB8);
    // fn apply(&mut self);
    // fn data(&mut self) -> &mut [RGB8; NUM_LEDS];
}

impl Canvas for [RGB8; NUM_LEDS] {
    fn line(&mut self, a: hex::Cube, b: hex::Cube, color: RGB8) {
        for c in hex::CubeLinedraw::new(a, b) {
            set_matrix_oddr(c.into(), color, self);
        }
    }

    fn clear(&mut self) {
        self.fill(color::BLACK);
    }
}
