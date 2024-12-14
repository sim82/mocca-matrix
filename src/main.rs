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
use embassy_rp::peripherals::{PIO0, PIO1, UART1};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::uart::{Async, Config, UartTx};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
// use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Ticker, Timer};
use mocca_matrix_embassy::i2s::{PioI2S, PioI2SProgram};
use mocca_matrix_embassy::power_zones::{DynamicLimit, NUM_ZONES};
use mocca_matrix_embassy::ws2812::{PioWs2812, PioWs2812Program};
use mocca_matrix_embassy::{power_zones, prelude::*};
use num_traits::{AsPrimitive, Saturating};
use smart_leds::{RGB, RGB8};
use {defmt_rtt as _, panic_probe as _};

const NUM_SAMPLES: usize = 64;
static SAMPLES: Signal<ThreadModeRawMutex, [i16; NUM_SAMPLES]> = Signal::new();

bind_interrupts!(struct Irqs0 {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

bind_interrupts!(struct Irqs1 {
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
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
    ws2812: PioWs2812<'static, PIO1, 1, NUM_LEDS>,
    count: u32,
}
impl LedStrip {
    pub fn new(ws2812: PioWs2812<'static, PIO1, 1, NUM_LEDS>) -> Self {
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
async fn rgb_soundmeter_task(mut ws2812: PioWs2812<'static, PIO1, 1, 8>) {
    let mut data = [RGB::default(); 8];
    // let mut led_strip = LedStrip::new(ws2812);
    let mut ticker = Ticker::every(Duration::from_millis(16));
    let mut i = 0u8;
    // let mut color = RGB::default();
    let mut smooth = 0i16;
    loop {
        if let Some(samples) = SAMPLES.try_take() {
            let min = samples.iter().min().unwrap();
            let max = samples.iter().max().unwrap();
            smooth = smooth.max(min.abs().max(max.abs()));
        }
        i = i.wrapping_add(1);
        // let f = (smooth / (i16::MAX / 32)).min(7);
        let f = (smooth.max(1).ilog2().saturating_sub(3)).min(8);
        info!("smooth: {} {}", smooth, f);
        data[0..(f as usize)].fill(RGB {
            r: (f * 16) as u8,
            g: ((8 - f) * 16) as u8,
            b: 0,
        });
        data[(f as usize)..].fill(RGB { r: 0, g: 0, b: 0 });

        ws2812.write(&data).await;
        smooth = smooth.saturating_sub((smooth / 8).max(1));
        ticker.next().await;
    }
}
#[embassy_executor::task]
async fn uart_task(mut uart_tx: UartTx<'static, UART1, Async>) {
    let mut samples = [0i16; 32 * 1024];
    loop {
        info!("receive");
        for c in samples.chunks_mut(32) {
            c.copy_from_slice(&SAMPLES.wait().await)
        }
        info!("send");
        let mut bytes = [0u8; 2048];
        for c in samples.chunks(1024) {
            // for (b, s) in bytes.iter_mut().zip(c) {
            //     *b = ((*s as u16) >> 8) as u8;
            // }
            for (i, s) in c.iter().enumerate() {
                bytes[i * 2] = ((*s as u16) & 0xff) as u8;
                bytes[i * 2 + 1] = (((*s as u16) >> 8) & 0xff) as u8;
                // byte_ptr = &mut byte_ptr[2..];
            }
            let _ = uart_tx.write(&bytes).await;
        }
    }
}
#[embassy_executor::task]
async fn rgb_task(ws2812: PioWs2812<'static, PIO1, 1, NUM_LEDS>) {
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

// #[embassy_executor::task]
// async fn i2s_sample_task(
//     mut i2s: PioI2S<'static, PIO0, 0>,
//     // mut uart_tx: UartTx<'static, UART1, Async>
// ) {
//     let mut words = [0u32; 32];
//     let mut samples = [0i32; 32];
//     let mut samples_16 = [0i16; 32];
//     let mut null = 0i32;
//     const N: i32 = 10i32;
//     loop {
//         // for _ in 0..20 {
//         i2s.read(&mut words).await;

//         for (o, i) in samples.iter_mut().zip(words) {
//             *o = ((i << 1) as i32) >> 14;
//         }
//         let avg = samples.iter().sum::<i32>() / samples.len() as i32;
//         null -= null / N;
//         null += avg / N;
//         // let samples_16 = &mut all_samples[i * 32..(i + 1) * 32];
//         for (o, i) in samples_16.iter_mut().zip(samples) {
//             *o = ((i - null) >> 2) as i16;
//             // *o = (i >> 2) as i16;
//         }

//         // let min = samples_16.iter().min().unwrap();
//         // let max = samples_16.iter().max().unwrap();

//         // minall = minall.min(*min);
//         // maxall = maxall.max(*max);
//         // AUDIO_LEVEL.signal(minall.abs().max(maxall.abs()));
//         SAMPLES.signal(samples_16);
//         info!("avg: {}", null);
//         // }
//     }
// }

#[embassy_executor::task]
async fn i2s_sample_task(
    mut i2s: PioI2S<'static, PIO0, 0>,
    // mut uart_tx: UartTx<'static, UART1, Async>
) {
    let mut words = [0u32; NUM_SAMPLES];
    let mut samples = [0i32; NUM_SAMPLES];
    let mut samples_16 = [0i16; NUM_SAMPLES];
    let mut null = 0i32;
    const N: i32 = 100i32;
    let mut filter = 0i32;
    let mut output = 0i32;
    let mut start = Instant::now();
    loop {
        // for _ in 0..20 {
        let dt = start.elapsed();
        let delay = dt.as_micros();
        i2s.read(&mut words).await;
        info!("delay: {}, trans: {}", delay, start.elapsed().as_micros());
        start = Instant::now();
        // info!("dma: {}", dt.as_micros());
        for (o, i) in samples.iter_mut().zip(words) {
            let new_val = ((i << 1) as i32) >> 14;
            // null -= null / N;
            // null += new_val / N;
            // hyper crappy high-pass
            null -= (null - new_val) / N;
            *o = new_val - null;
            // *o *= 64;
        }
        let avg = samples.iter().sum::<i32>() / samples.len() as i32;
        // let samples_16 = &mut all_samples[i * 32..(i + 1) * 32];
        for (o, i) in samples_16.iter_mut().zip(samples) {
            // *o = ((i - null) >> 2) as i16;
            *o = (i >> 2) as i16;
            // *o = (i) as i16;
        }

        // let min = samples_16.iter().min().unwrap();
        // let max = samples_16.iter().max().unwrap();

        // minall = minall.min(*min);
        // maxall = maxall.max(*max);
        // AUDIO_LEVEL.signal(minall.abs().max(maxall.abs()));
        SAMPLES.signal(samples_16);

        // info!("avg: {}", null);
        // }
    }
}
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let led = Output::new(p.PIN_25, Level::Low);

    let i2s = {
        let Pio {
            mut common, sm0, ..
        } = Pio::new(p.PIO0, Irqs0);
        let program = PioI2SProgram::new2(&mut common);
        PioI2S::new(
            &mut common,
            sm0,
            p.DMA_CH0,
            p.PIN_14,
            p.PIN_15,
            p.PIN_16,
            &program,
        )
    };

    unwrap!(spawner.spawn(i2s_sample_task(i2s /*, uart_tx*/)));
    /////////////////////////////
    let ws2812 = {
        let Pio {
            mut common, sm1, ..
        } = Pio::new(p.PIO1, Irqs1);
        let program = PioWs2812Program::new(&mut common);
        PioWs2812::new(&mut common, sm1, p.DMA_CH1, p.PIN_17, &program)
    };
    // unwrap!(spawner.spawn(rgb_task(ws2812)));
    unwrap!(spawner.spawn(rgb_soundmeter_task(ws2812)));

    /////////////////////////////
    // let mut uart_tx = UartTx::new(p.UART1, p.PIN_8, p.DMA_CH3, Config::default());
    // unwrap!(spawner.spawn(uart_task(uart_tx)));

    /////////////////////////////
    unwrap!(spawner.spawn(blink_task(led)));
}
