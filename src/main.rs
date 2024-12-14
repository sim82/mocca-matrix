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
use cortex_m_rt::entry;
use defmt::*;
use embassy_executor::{Executor, InterruptExecutor, Spawner};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::interrupt::{InterruptExt, Priority};
use embassy_rp::peripherals::{PIO0, PIO1, UART1};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::uart::{Async, Config, UartTx};
use embassy_rp::{bind_interrupts, interrupt};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
// use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Ticker, Timer, TICK_HZ};
use idsp::iir::Biquad;
use mocca_matrix_embassy::i2s::{PioI2S, PioI2SProgram};
use mocca_matrix_embassy::power_zones::{DynamicLimit, NUM_ZONES};
use mocca_matrix_embassy::ws2812::{self, PioWs2812, PioWs2812Program};
use mocca_matrix_embassy::{power_zones, prelude::*};
use num_traits::{AsPrimitive, Saturating, WrappingAdd};
use smart_leds::{RGB, RGB8};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

const NUM_SAMPLES: usize = 64;
static SAMPLES: Signal<CriticalSectionRawMutex, [i16; NUM_SAMPLES]> = Signal::new();

static LEDS: Signal<CriticalSectionRawMutex, [RGB8; NUM_LEDS]> = Signal::new();

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
    count: u32,
}
impl LedStrip {
    pub fn new() -> Self {
        Self {
            data: [RGB8::default(); NUM_LEDS],
            dynamic_limit: Default::default(),
            count: 0,
        }
    }
    pub async fn signal(&mut self) {
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
        LEDS.signal(self.data);
    }
}
// const NUM_LEDS: usize = 8;
#[embassy_executor::task]
async fn rgb_soundmeter_task() {
    let mut data = [RGB::default(); NUM_LEDS];
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
        let f = (smooth.max(1).ilog2().saturating_sub(1)).min(8);
        info!("smooth: {} {}", smooth, f);
        data.fill(RGB {
            r: (f * 16) as u8,
            g: ((8 - f) * 16) as u8,
            b: 0,
        });
        LEDS.signal(data);
        // data[0..(f as usize)].fill(RGB {
        //     r: (f * 16) as u8,
        //     g: ((8 - f) * 16) as u8,
        //     b: 0,
        // });
        // data[(f as usize)..].fill(RGB { r: 0, g: 0, b: 0 });

        // ws2812.write(&data).await;
        smooth = smooth.saturating_sub((smooth / 16).max(1));
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
async fn rgb_task() {
    let mut led_strip = LedStrip::new();
    let mut ticker = Ticker::every(Duration::from_millis(16));
    let mut splash = app::drawing::new();
    let mut app = app::hexlife2::new();
    // let mut app = app::power::new();
    let mut app = app::cellular::new();
    loop {
        let start = Instant::now();
        app.tick(&mut led_strip.data);
        let dt = start.elapsed();

        // info!("calc: {}", dt.as_micros());
        let start = Instant::now();

        led_strip.signal().await;
        // info!("write: {}", start.elapsed().as_micros());
        let start = Instant::now();
        ticker.next().await;
        // info!("wait: {}", start.elapsed().as_micros());
    }
}

#[embassy_executor::task]
async fn rgb_simple() {
    let mut ticker = Ticker::every(Duration::from_millis(16));
    let mut i = 0;
    let mut data = [RGB {
        r: 0u8,
        g: 0u8,
        b: 0u8,
    }; NUM_LEDS];
    loop {
        ticker.next().await;

        data.fill(RGB { r: i, g: 0, b: 0 });
        LEDS.signal(data);
        i = i.wrapping_add(1);
    }
}

#[embassy_executor::task]
async fn rgb_writer_task(mut ws2812: PioWs2812<'static, PIO1, 1, NUM_LEDS>) {
    loop {
        let leds = LEDS.wait().await;
        ws2812.write(&leds).await;
    }
}

#[embassy_executor::task]
async fn i2s_sample_task(
    mut i2s: PioI2S<'static, PIO0, 0>,
    // mut uart_tx: UartTx<'static, UART1, Async>
) {
    let mut words = [0u32; NUM_SAMPLES];
    let mut samples = [0i32; NUM_SAMPLES];
    let mut samples_16 = [0i16; NUM_SAMPLES];
    let mut null = 0i32;
    const N: i32 = 16i32;
    let mut filter = 0i32;
    let mut output = 0i32;
    let mut start = Instant::now();
    let coeff = &[
        0.9972549369794074,
        -1.9945098739588147,
        0.9972549369794074,
        1.0,
        -1.9944845691862039,
        0.9945351787314256,
    ];

    let filter: Biquad<i32> = coeff.into();
    let mut xy = [0i32; 4];
    // let mut filter = Biquad::
    loop {
        // for _ in 0..20 {
        i2s.read(&mut words).await;
        // info!("dma: {}", dt.as_micros());
        for (o, i) in samples.iter_mut().zip(words) {
            let new_val = ((i << 1) as i32) >> 14;
            // *o = filter.update(&mut xy, new_val);
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
#[embassy_executor::task]
async fn run_high() {
    loop {
        info!("        [high] tick!");
        Timer::after_ticks(673740).await;
    }
}

#[embassy_executor::task]
async fn run_low() {
    loop {
        let start = Instant::now();
        info!("[low] Starting long computation");

        // Spin-wait to simulate a long CPU computation
        embassy_time::block_for(embassy_time::Duration::from_secs(2)); // ~2 seconds

        let end = Instant::now();
        let ms = end.duration_since(start).as_ticks() * 1000 / TICK_HZ;
        info!("[low] done in {} ms", ms);

        Timer::after_ticks(82983).await;
    }
}

static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();
static EXECUTOR_LOW: StaticCell<Executor> = StaticCell::new();

#[interrupt]
unsafe fn SWI_IRQ_1() {
    EXECUTOR_HIGH.on_interrupt()
}

#[entry]
fn main() -> ! {
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

    // High-priority executor: SWI_IRQ_1, priority level 2
    interrupt::SWI_IRQ_1.set_priority(Priority::P2);
    let spawner_high = EXECUTOR_HIGH.start(interrupt::SWI_IRQ_1);
    // unwrap!(spawner_high.spawn(run_high()));
    // unwrap!(spawner.spawn(run_med()));
    unwrap!(spawner_high.spawn(i2s_sample_task(i2s /*, uart_tx*/)));
    /////////////////////////////
    let ws2812 = {
        let Pio {
            mut common, sm1, ..
        } = Pio::new(p.PIO1, Irqs1);
        let program = PioWs2812Program::new(&mut common);
        PioWs2812::new(&mut common, sm1, p.DMA_CH1, p.PIN_17, &program)
    };

    /////////////////////////////
    // let mut uart_tx = UartTx::new(p.UART1, p.PIN_8, p.DMA_CH3, Config::default());
    // unwrap!(spawner.spawn(uart_task(uart_tx)));

    /////////////////////////////
    // Low priority executor: runs in thread mode, using WFE/SEV
    let executor = EXECUTOR_LOW.init(Executor::new());
    executor.run(|spawner| {
        unwrap!(spawner.spawn(blink_task(led)));
        unwrap!(spawner.spawn(rgb_task()));
        // unwrap!(spawner.spawn(rgb_simple()));
        // unwrap!(spawner.spawn(rgb_soundmeter_task()));
        // unwrap!(spawner.spawn(run_low()));
        unwrap!(spawner.spawn(rgb_writer_task(ws2812 /*, uart_tx*/)));
    });
}
