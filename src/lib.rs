#![no_std]
pub use smart_leds::RGB8;
pub mod app;
pub mod bitzet;
pub mod canvas;
pub mod color;
pub mod effects;
pub mod hex;
pub mod math;
pub mod matrix;
pub mod power_zones;
pub mod ws2812;

pub mod prelude {
    pub use super::{
        app,
        canvas,
        color,
        color::Rainbow,
        color::HV8,
        matrix::adjacent,
        matrix::get_matrix,
        matrix::led_addr,
        // app, canvas::Canvas, color, color::Rainbow, effects, get_matrix, hal, power_zones,
        // set_matrix, set_matrix_oddr, Console,
        matrix::set_matrix,
        matrix::set_matrix_oddr,
        matrix::Error,
        matrix::MATRIX_HEIGHT,
        matrix::MATRIX_WIDTH,
        matrix::NUM_LEDS,
        RGB8,
    };
}
