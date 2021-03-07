use smart_leds::brightness;

use crate::math::Vec2;
use crate::prelude::*;
use crate::{color::WHITE, hex::prelude::*};
use micromath::F32Ext;

pub struct Drawing {
    i: u32,
}

impl app::App for Drawing {
    fn new() -> Self {
        Drawing { i: 0 }
    }

    fn tick(&mut self, canvas: &mut [RGB8; NUM_LEDS]) {
        if self.i >= (360 / 6) {
            self.i = 0;
        }
        let i = self.i;
        // canvas.clear();
        canvas.iter_mut().for_each(|v| {
            *v = brightness(core::iter::once(*v), 210).next().unwrap();
        });
        let f = ((i * 6) as f32).to_radians();
        let s = f.sin();
        let c = f.cos();
        // let (sin, cos) = f.sin();
        // f.sin()
        // let v0 = Vec2::new((s * -5f32) as i32, (c * -5f32) as i32);
        let v0 = Cube::zero();
        let v = Vec2::new((s * 15f32) as i32, (c * 15f32) as i32);

        canvas.line(v0.into(), v.into(), color::WHITE);
        self.i += 1;
        // canvas.data()[1] = RGB8::default();
        // periphery.delay.delay_ms(8u8);
        // let v =
    }
}
