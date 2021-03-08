use crate::prelude::*;
use crate::{color::WHITE, hex::prelude::*};

pub struct Power {
    i: usize,
    on : [bool; NUM_LEDS]
}

impl app::App for Power {
    fn new() -> Self {
        Power { i: 0, on : [false; NUM_LEDS] }
    }

    fn tick(&mut self, canvas: &mut [RGB8; NUM_LEDS]) {
        // canvas.fill(color::BLACK);
            let i = self.i % NUM_LEDS;
        // for i in 0..(self.i % NUM_LEDS) {
            if !self.on[i] { 
                canvas[i] = color::WHITE;
            } else {
                canvas[i] = color::BLACK;
            }
            self.on[i] = !self.on[i];
        // }
        self.i += 1;
    }
}
