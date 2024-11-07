#![no_main]
#![no_std]

extern crate panic_halt;

extern crate stm32l4xx_hal as hal;

use mocca_matrix_rtic::{app::App, hex::Hex, prelude::*};
use smart_leds::{brightness, RGB8};
use ws2812::Ws2812;

use core::fmt::Write;
use embedded_graphics::{fonts, pixelcolor, prelude::*, style};
use hal::{
    device::I2C1,
    gpio::gpioa::PA0,
    gpio::{
        Alternate, Edge, Floating, Input, OpenDrain, Output, PullUp, PushPull, PA1, PA10, PA5, PA6,
        PA7, PA8, PA9, PB6, PB7, PB8, PB9,
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

const REFRESH_DISPLAY_PERIOD: u32 = 64_000_000 / 20;
const REFRESH_LED_STRIP_PERIOD: u32 = 64_000_000 / 60;

pub struct Sai {
    lrclk: PA9<Alternate<hal::gpio::AF13, Output<PushPull>>>,
    bclk_out: PA8<Alternate<hal::gpio::AF13, Output<PushPull>>>,
    data_in: PA10<Alternate<hal::gpio::AF13, Input<Floating>>>,
}

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

        dbg_pin: PA1<Output<PushPull>>,

        sai: Sai,
    }

    #[init(schedule = [refresh_display])]
    fn init(mut cx: init::Context) -> init::LateResources {
        cx.device.RCC.cr.write(|w| w.pllsai1on().clear_bit());
        while cx.device.RCC.cr.read().pllsai1rdy().bit_is_set() {}
        cx.device.RCC.pllsai1cfgr.write(|w| unsafe {
            w.pllsai1pen()
                .set_bit()
                //  .pllsai1p()
                //  .bits(2)
                .pllsai1n()
                .bits(8)
        });
        cx.device.RCC.cr.write(|w| w.pllsai1on().set_bit());
        cx.device
            .RCC
            .ccipr
            .write(|w| unsafe { w.sai1sel().bits(0b10) });

        //  dp.RCC.pllsai1cfgr.write(|w| w.pllsai().;
        cx.device.RCC.apb2enr.write(|w| w.sai1en().set_bit());
        // 2. reset it
        cx.device.RCC.apb2rstr.write(|w| w.sai1rst().set_bit());
        cx.device.RCC.apb2rstr.write(|w| w.sai1rst().clear_bit());

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

        // =========================================================================
        // setup dbg pin
        let mut gpioa = cx.device.GPIOA.split(&mut rcc.ahb2);

        let mut dbg_pin = gpioa
            .pa1
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

        // ==========================================================================
        // setup SAI1 A
        dbg_pin.set_high().ok();
        dbg_pin.set_low().ok();
        let mut lrclk = gpioa
            .pa9
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        let lrclk = lrclk.into_af13(&mut gpioa.moder, &mut gpioa.afrh);

        dbg_pin.set_high().ok();
        dbg_pin.set_low().ok();
        let mut bclk_out = gpioa
            .pa8
            // .into_open_drain_output(&mut gpioa.moder, &mut gpioa.otyper);
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        // bclk_out.set_high().ok();
        // bclk_out.set_low().ok();
        let bclk_out = bclk_out.into_af13(&mut gpioa.moder, &mut gpioa.afrh);

        let mut data_in = gpioa
            .pa10
            .into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr);
        let data_in = data_in.into_af13(&mut gpioa.moder, &mut gpioa.afrh);

        // dbg_pin.set_high().ok();
        // dbg_pin.set_low().ok();
        // let mut data_out = gpioa
        //     .pa10
        //     .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        // let data_out = data_out.into_af13(&mut gpioa.moder, &mut gpioa.afrh);
        let mut mclk = gpioa
            .pa3
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        let mclk = mclk.into_af13(&mut gpioa.moder, &mut gpioa.afrl);

        cx.device.SAI1.cha.frcr.write(|w| unsafe {
            w
                //.fspol()            .rising_edge() // FS is active high
                .fsdef()
                .set_bit() // FS is start of frame and channel indication
                .fsall()
                .bits(31) // FS high for half frame
                .frl()
                .bits(63) // frame is 32bits
        });

        // setup slotr
        cx.device.SAI1.cha.slotr.write(|w| unsafe {
            w.sloten()
                .bits(0b11) // enable slots 0, 1
                .nbslot()
                .bits(1) // two slots
                .slotsz()
                .bit32() // 32bit per slot
        });
        dbg_pin.set_high().ok();
        dbg_pin.set_low().ok();

        // cx.device
        //     .SAI1
        //     .cha
        //     .dr
        //     .write(|w| unsafe { w.data().bits(0b1010101010101011) });

        // setup CR and enable
        cx.device.SAI1.cha.cr1.write(|w| unsafe {
            w.lsbfirst()
                .msb_first() // big endian
                .ds()
                .bit16() // DS = 16bit
                .ckstr()
                .rising_edge()
                .mode()
                .master_rx() // master rx
                .prtcfg()
                .free()
                .mckdiv()
                .bits(4)
                .saien()
                .enabled()
        });
        dbg_pin.set_high().ok();
        dbg_pin.set_low().ok();

        while !cx.device.SAI1.cha.sr.read().flvl().is_full() {
            dbg_pin.set_high().ok();
            dbg_pin.set_low().ok();
        }
        // Initialization of late resources
        init::LateResources {
            timer,
            disp,
            dbg_pin,
            sai: Sai {
                lrclk,
                bclk_out,
                data_in,
            },
        }
    }

    #[task(schedule=[refresh_display], resources = [disp], priority = 1)]
    fn refresh_display(mut cx: refresh_display::Context) {
        let mut text = String::<U32>::new();

        text.clear();
        write!(&mut text, "{:?}", cx.scheduled).unwrap();
        cx.resources.disp.write(&text, Some(5));
        cx.resources.disp.flush().unwrap();
        cx.schedule
            .refresh_display(cx.scheduled + REFRESH_DISPLAY_PERIOD.cycles())
            .unwrap();
    }

    extern "C" {
        fn COMP();
        fn SDMMC1();
    }
};
