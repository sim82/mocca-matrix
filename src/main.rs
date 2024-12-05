// //! This example test the RP Pico on board LED.
// //!
// //! It does not work with the RP Pico W board. See wifi_blinky.rs.

// #![no_std]
// #![no_main]

// use defmt::*;
// use embassy_executor::Spawner;
// use embassy_rp::gpio;
// use embassy_time::Timer;
// use gpio::{Level, Output};
// use {defmt_rtt as _, panic_probe as _};

// #[embassy_executor::main]
// async fn main(_spawner: Spawner) {
//     let p = embassy_rp::init(Default::default());
//     let mut led = Output::new(p.PIN_25, Level::Low);

//     loop {
//         info!("led on!");
//         led.set_high();
//         Timer::after_secs(1).await;

//         info!("led off!");
//         led.set_low();
//         Timer::after_secs(1).await;
//     }
// }

//! This example shows powerful PIO module in the RP2040 chip to communicate with WS2812 LED modules.
//! See (https://www.sparkfun.com/categories/tags/ws2812)

#![no_std]
#![no_main]

use app::App;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
// use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use embassy_time::{Duration, Instant, Ticker, Timer};
use mocca_matrix_embassy::i2s::{PioI2S, PioI2SProgram};
use mocca_matrix_embassy::power_zones::{DynamicLimit, NUM_ZONES};
use mocca_matrix_embassy::ws2812::{PioWs2812, PioWs2812Program};
use mocca_matrix_embassy::{power_zones, prelude::*};
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(100).await;
        led.set_low();
        Timer::after_millis(100).await;
    }
}
pub struct LedStrip {
    pub data: [RGB8; NUM_LEDS],
    dynamic_limit: [DynamicLimit; NUM_ZONES],
    ws2812: PioWs2812<'static, PIO0, 0, NUM_LEDS>,
    count: u32,
}
impl LedStrip {
    pub fn new(ws2812: PioWs2812<'static, PIO0, 0, NUM_LEDS>) -> Self {
        Self {
            data: [RGB8::default(); NUM_LEDS],
            dynamic_limit: Default::default(),
            count: 0,
            ws2812,
        }
    }
    pub async fn write(&mut self) {
        let led_strip_power = power_zones::estimate_current_all(&self.data);
        let mut limit = [0u32; NUM_ZONES];
        for i in 0..NUM_ZONES {
            self.dynamic_limit[i].add_measurement(led_strip_power[i]);
            if self.count % 32 == 0 {
                self.dynamic_limit[i].commit();
            }
            limit[i] = self.dynamic_limit[i].get_limit();
        }

        // info!("power: {:?} {:?}", led_strip_power, limit);
        power_zones::limit_current(&mut self.data, &limit);
        self.count = self.count.wrapping_add(1);
        self.ws2812.write(&self.data).await;
    }
}
// const NUM_LEDS: usize = 8;
#[embassy_executor::task]
async fn rgb_task(ws2812: PioWs2812<'static, PIO0, 0, NUM_LEDS>) {
    let mut led_strip = LedStrip::new(ws2812);
    let mut ticker = Ticker::every(Duration::from_millis(16));
    let mut splash = app::drawing::new();
    let mut app = app::hexlife2::new();
    // let mut app = app::power::new();
    let mut app = app::cellular::new();
    loop {
        let start = Instant::now();
        app.tick(&mut led_strip.data);
        let dt = start.elapsed();

        info!("calc: {}", dt.as_micros());
        let start = Instant::now();
        led_strip.write().await;
        info!("write: {}", start.elapsed().as_micros());
        let start = Instant::now();
        ticker.next().await;
        info!("wait: {}", start.elapsed().as_micros());
    }
}
#[embassy_executor::task]
async fn i2s_task(i2s: PioI2S<'static, PIO0, 0, NUM_LEDS>) {
    let mut ticker = Ticker::every(Duration::from_millis(16));
    loop {
        //
        ticker.next().await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let led = Output::new(p.PIN_25, Level::Low);

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    // Common neopixel pins:
    // Thing plus: 8
    // Adafruit Feather: 16;  Adafruit Feather+RFM95: 4
    // let program = PioWs2812Program::new(&mut common);
    // let ws2812 = PioWs2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_16, &program);
    let program = PioI2SProgram::new2(&mut common);
    let i2s = PioI2S::new(&mut common, sm0, p.DMA_CH0, p.PIN_14, p.PIN_15, &program);

    // Loop forever making RGB values and pushing them out to the WS2812.
    unwrap!(spawner.spawn(blink_task(led)));
    unwrap!(spawner.spawn(i2s_task(i2s)));
    // unwrap!(spawner.spawn(rgb_task(ws2812)));
}
