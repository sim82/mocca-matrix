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
use embassy_time::{Duration, Ticker, Timer};
use mocca_matrix_embassy::power_zones::{DynamicLimit, NUM_ZONES};
use mocca_matrix_embassy::ws2812::{PioWs2812, PioWs2812Program};
use mocca_matrix_embassy::{power_zones, prelude::*};
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

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

        info!("power: {:?} {:?}", led_strip_power, limit);
        power_zones::limit_current(&mut self.data, &limit);
        self.count = self.count.wrapping_add(1);
        self.ws2812.write(&self.data).await;
    }
}
// const NUM_LEDS: usize = 8;
#[embassy_executor::task]
async fn rgb_task(mut ws2812: PioWs2812<'static, PIO0, 0, NUM_LEDS>) {
    let mut led_strip = LedStrip::new(ws2812);
    // This is the number of leds in the string. Helpfully, the sparkfun thing plus and adafruit
    // feather boards for the 2040 both have one built in.
    // let mut data = [RGB8::default(); NUM_LEDS];
    let mut ticker = Ticker::every(Duration::from_millis(16));
    // let mut dynamic_limit = [DynamicLimit::default(); NUM_ZONES];
    // let mut count = 0u32;
    // loop {
    //     for j in 0..(256 * 5) {
    //         // debug!("New Colors:");
    //         for i in 0..NUM_LEDS {
    //             led_strip.data[i] =
    //                 wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
    //             // debug!("R: {} G: {} B: {}", data[i].r, data[i].g, data[i].b);
    //         }

    //         led_strip.write().await;

    //         ticker.next().await;
    //     }
    // }
    // let mut app = app::drawing::new();
    let mut app = app::hexlife2::new();
    loop {
        // led_strip.data.fill([255, 255, 255].into());

        app.tick(&mut led_strip.data);
        led_strip.write().await;
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
    let program = PioWs2812Program::new(&mut common);
    let ws2812 = PioWs2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_16, &program);

    // Loop forever making RGB values and pushing them out to the WS2812.
    unwrap!(spawner.spawn(blink_task(led)));
    unwrap!(spawner.spawn(rgb_task(ws2812)));
}
