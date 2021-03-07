use crate::prelude::*;

pub struct Hexlife {
    rainbow: Rainbow,
}

impl app::App for Hexlife {
    fn new() -> Self {
        Hexlife {
            rainbow: Rainbow::step(1),
        }
    }

    fn tick(&mut self, led_data: &mut [RGB8; NUM_LEDS]) {
        led_data
            .iter_mut()
            .for_each(|c| *c = self.rainbow.next().unwrap());
    }
}
