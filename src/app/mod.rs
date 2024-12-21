use crate::prelude::*;

pub mod drawing;
// pub mod hexlife;
pub mod cellular;
pub mod hexlife2;
pub mod power;

#[derive(Default, Clone)]
pub struct Env {
    pub spl_db: f32,
}

pub trait App {
    // fn new() -> Self;
    fn tick(&mut self, led_data: &mut [RGB8; NUM_LEDS], env: &Env);
}
