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
        app::{self, App},
        canvas::{self, Canvas},
        color::{self, Rainbow, HV8},
        math::Vec2,
        matrix::{
            adjacent, get_matrix, led_addr, set_matrix, set_matrix_oddr, Error, MATRIX_HEIGHT,
            MATRIX_WIDTH, NUM_LEDS,
        },
        RGB8,
    };
}
