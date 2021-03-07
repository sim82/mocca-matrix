#![no_main]
#![no_std]

extern crate panic_halt;

extern crate stm32l4xx_hal as hal;

use mocca_matrix_rtic::{app::App, prelude::*};
use smart_leds::{brightness, RGB8};
use ws2812::Ws2812;

use core::fmt::Write;
use embedded_graphics::{fonts, pixelcolor, prelude::*, style};
use hal::{
    device::I2C1,
    gpio::gpioa::PA0,
    gpio::{
        Alternate, Edge, Floating, Input, OpenDrain, Output, PullUp, PushPull, PA1, PA5, PA6, PA7,
        PB6, PB7, PB8, PB9,
    },
    i2c::I2c,
    prelude::*,
    spi::Spi,
    stm32,
    timer::{Event, Timer},
};
use hal::{
    gpio::PC13,
    stm32l4::stm32l4x2::{interrupt, Interrupt, NVIC},
};
use heapless::consts::*;
use heapless::String;
use rtic::cyccnt::U32Ext;
use smart_leds::SmartLedsWrite;
use ssd1306::{mode::GraphicsMode, prelude::*, Builder, I2CDIBuilder};
use ws2812_spi as ws2812;

const REFRESH_DISPLAY_PERIOD: u32 = 64_000_000 / 40;
const REFRESH_LED_STRIP_PERIOD: u32 = 64_000_000 / 32;

#[rtic::app(device = hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        timer: Timer<stm32::TIM7>,
        disp: GraphicsMode<
            I2CInterface<
                I2c<
                    I2C1,
                    (
                        PB6<Alternate<hal::gpio::AF4, Output<OpenDrain>>>,
                        PB7<Alternate<hal::gpio::AF4, Output<OpenDrain>>>,
                    ),
                >,
            >,
            DisplaySize128x64,
        >,
        led_strip_dev: ws2812_spi::Ws2812<
            Spi<
                hal::pac::SPI1,
                (
                    PA5<Alternate<hal::gpio::AF5, Input<Floating>>>,
                    PA6<Alternate<hal::gpio::AF5, Input<Floating>>>,
                    PA7<Alternate<hal::gpio::AF5, Input<Floating>>>,
                ),
            >,
        >,
        led_strip_data: [RGB8; NUM_LEDS],
        led_strip_current: [u32; 4],
        dynamic_limit: [power_zones::DynamicLimit; 4],
        app: crate::app::drawing::Drawing,
        count: u32,
        dbg_pin: PA1<Output<PushPull>>,
    }

    #[init(schedule = [refresh_display, refresh_led_strip])]
    fn init(mut cx: init::Context) -> init::LateResources {
        let mut rcc = cx.device.RCC.constrain();
        let mut flash = cx.device.FLASH.constrain();
        let mut pwr = cx.device.PWR.constrain(&mut rcc.apb1r1);
        let mut cp = cx.core;

        // software tasks won't work without this:
        cp.DCB.enable_trace();
        cp.DWT.enable_cycle_counter();

        let clocks = rcc
            .cfgr
            .sysclk(64.mhz())
            .pclk1(16.mhz())
            .pclk2(64.mhz())
            .freeze(&mut flash.acr, &mut pwr);

        // ================================================================================
        // Set up Timer interrupt
        let mut timer = Timer::tim7(cx.device.TIM7, 4.khz(), clocks, &mut rcc.apb1r1);
        timer.listen(Event::TimeOut);

        // ================================================================================
        // set up OLED i2c
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.ahb2);
        let mut scl = gpiob
            .pb6
            .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
        scl.internal_pull_up(&mut gpiob.pupdr, true);
        let scl = scl.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
        let mut sda = gpiob
            .pb7
            .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
        sda.internal_pull_up(&mut gpiob.pupdr, true);
        let sda = sda.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

        let mut i2c = I2c::i2c1(
            cx.device.I2C1,
            (scl, sda),
            800.khz(),
            clocks,
            &mut rcc.apb1r1,
        );

        let interface = I2CDIBuilder::new().init(i2c);
        let mut disp: GraphicsMode<_, _> = Builder::new()
            // .with_size(DisplaySize::Display128x64NoOffset)
            .connect(interface)
            .into();
        disp.init().unwrap();
        disp.flush().unwrap();

        disp.write("hello world xxx!", None);
        disp.flush().unwrap();
        cx.schedule
            .refresh_display(cx.start + REFRESH_DISPLAY_PERIOD.cycles())
            .unwrap();

        // ================================================================================
        // setup smart-led strip
        let mut gpioa = cx.device.GPIOA.split(&mut rcc.ahb2);
        let (sck, miso, mosi) = {
            (
                gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl),
                gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl),
                gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl),
            )
        };

        // Configure SPI with 3Mhz rate
        let spi = Spi::spi1(
            cx.device.SPI1,
            (sck, miso, mosi),
            ws2812::MODE,
            3_000_000.hz(),
            clocks,
            &mut rcc.apb2,
        );
        let led_strip_dev = Ws2812::new(spi);

        cx.schedule
            .refresh_led_strip(cx.start + REFRESH_LED_STRIP_PERIOD.cycles())
            .unwrap();

        let dbg_pin = gpioa
            .pa1
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

        // Initialization of late resources
        init::LateResources {
            timer,
            disp,
            led_strip_dev,
            led_strip_data: [mocca_matrix_rtic::color::BLACK; NUM_LEDS],
            led_strip_current: [0; 4],
            dynamic_limit: Default::default(),
            app: crate::app::drawing::Drawing::new(),
            count: 0,
            dbg_pin,
        }
    }

    #[task(schedule=[refresh_display], resources = [disp, dynamic_limit, led_strip_current], priority = 1)]
    fn refresh_display(mut cx: refresh_display::Context) {
        let mut text = String::<U32>::new();

        let a = cx.resources.led_strip_current.lock(|x| x.clone());
        let limit_dyn: heapless::Vec<u32, U4> = cx
            .resources
            .dynamic_limit
            .lock(|x| x.iter().map(|limit| limit.get_limit()).collect());

        for (i, (c, limit_dyn)) in a.iter().zip(limit_dyn.iter()).enumerate() {
            text.clear();

            write!(&mut text, "I({}): {} {}", i, c, limit_dyn).unwrap();
            cx.resources.disp.write(&text, Some(i as i32));
        }

        text.clear();
        write!(&mut text, "{:?}", cx.scheduled).unwrap();
        cx.resources.disp.write(&text, Some(5));
        cx.resources.disp.flush().unwrap();
        cx.schedule
            .refresh_display(cx.scheduled + REFRESH_DISPLAY_PERIOD.cycles())
            .unwrap();
    }
    #[task(schedule=[refresh_led_strip], resources = [led_strip_dev, led_strip_data, led_strip_current, dynamic_limit, app, count, dbg_pin], priority = 3)]
    fn refresh_led_strip(mut cx: refresh_led_strip::Context) {
        // let mut rainbow = brightness(cx.resources.rainbow, 64);
        cx.resources.dbg_pin.set_low().ok();

        cx.resources.app.tick(cx.resources.led_strip_data);

        cx.resources.dbg_pin.set_high().ok();

        *cx.resources.led_strip_current =
            power_zones::estimate_current_all(cx.resources.led_strip_data);

        cx.resources.dbg_pin.set_low().ok();

        let mut limit = [0u32; power_zones::NUM_ZONES];
        for i in 0..power_zones::NUM_ZONES {
            cx.resources.dynamic_limit[i].add_measurement(cx.resources.led_strip_current[i]);
            if *cx.resources.count % 32 == 0 {
                cx.resources.dynamic_limit[i].commit();
            }
            limit[i] = cx.resources.dynamic_limit[i].get_limit();
        }

        cx.resources.dbg_pin.set_high().ok();
        power_zones::limit_current(&mut cx.resources.led_strip_data, &limit);

        cx.resources.dbg_pin.set_low().ok();
        cx.resources
            .led_strip_dev
            .write(cx.resources.led_strip_data.iter().cloned())
            .unwrap();

        cx.resources.dbg_pin.set_high().ok();
        cx.schedule
            .refresh_led_strip(cx.scheduled + REFRESH_LED_STRIP_PERIOD.cycles())
            .unwrap();

        *cx.resources.count = cx.resources.count.overflowing_add(1).0;
    }

    extern "C" {
        fn COMP();
        fn SDMMC1();
    }
};
