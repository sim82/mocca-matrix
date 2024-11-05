#![no_std]
pub mod ws2812 {
    //! [ws2812](https://www.sparkfun.com/datasheets/LCD/HD44780.pdf)

    use embassy_time::Timer;
    use fixed::types::U24F8;
    use smart_leds::RGB8;

    use embassy_rp::clocks::clk_sys_freq;
    use embassy_rp::dma::{AnyChannel, Channel};
    use embassy_rp::pio::{
        Common, Config, FifoJoin, Instance, LoadedProgram, PioPin, ShiftConfig, ShiftDirection,
        StateMachine,
    };
    use embassy_rp::{into_ref, Peripheral, PeripheralRef};

    const T1: u8 = 2; // start bit
    const T2: u8 = 5; // data bit
    const T3: u8 = 3; // stop bit
    const CYCLES_PER_BIT: u32 = (T1 + T2 + T3) as u32;

    /// This struct represents a ws2812 program loaded into pio instruction memory.
    pub struct PioWs2812Program<'a, PIO: Instance> {
        prg: LoadedProgram<'a, PIO>,
    }

    impl<'a, PIO: Instance> PioWs2812Program<'a, PIO> {
        /// Load the ws2812 program into the given pio
        pub fn new2(common: &mut Common<'a, PIO>) -> Self {
            let prg = pio_proc::pio_asm!(
                r#"
                    .side_set 1
                    set pindirs, 1               side 0
                    .wrap_target
                    start:
                        out x, 1 [2]             side 0
                        jmp !x, do_zero [1]      side 1
                        jmp start [4]            side 1
                    do_zero:
                        nop [4]                  side 0
                    .wrap
                "#
            );
            let prg = common.load_program(&prg.program);
            Self { prg }
        }
        pub fn new(common: &mut Common<'a, PIO>) -> Self {
            let side_set = pio::SideSet::new(false, 1, false);
            let mut a: pio::Assembler<32> = pio::Assembler::new_with_side_set(side_set);

            let mut wrap_target = a.label();
            let mut wrap_source = a.label();
            let mut do_zero = a.label();
            a.set_with_side_set(pio::SetDestination::PINDIRS, 1, 0);
            a.bind(&mut wrap_target);
            // Do stop bit
            a.out_with_delay_and_side_set(pio::OutDestination::X, 1, T3 - 1, 0);
            // Do start bit
            a.jmp_with_delay_and_side_set(pio::JmpCondition::XIsZero, &mut do_zero, T1 - 1, 1);
            // Do data bit = 1
            a.jmp_with_delay_and_side_set(pio::JmpCondition::Always, &mut wrap_target, T2 - 1, 1);
            a.bind(&mut do_zero);
            // Do data bit = 0
            a.nop_with_delay_and_side_set(T2 - 1, 0);
            a.bind(&mut wrap_source);

            let prg = a.assemble_with_wrap(wrap_source, wrap_target);
            let prg = common.load_program(&prg);

            Self { prg }
        }
    }

    /// Pio backed ws2812 driver
    /// Const N is the number of ws2812 leds attached to this pin
    pub struct PioWs2812<'d, P: Instance, const S: usize, const N: usize> {
        dma: PeripheralRef<'d, AnyChannel>,
        sm: StateMachine<'d, P, S>,
    }

    impl<'d, P: Instance, const S: usize, const N: usize> PioWs2812<'d, P, S, N> {
        /// Configure a pio state machine to use the loaded ws2812 program.
        pub fn new(
            pio: &mut Common<'d, P>,
            mut sm: StateMachine<'d, P, S>,
            dma: impl Peripheral<P = impl Channel> + 'd,
            pin: impl PioPin,
            program: &PioWs2812Program<'d, P>,
        ) -> Self {
            into_ref!(dma);

            // Setup sm0
            let mut cfg = Config::default();

            // Pin config
            let out_pin = pio.make_pio_pin(pin);
            cfg.set_out_pins(&[&out_pin]);
            cfg.set_set_pins(&[&out_pin]);

            cfg.use_program(&program.prg, &[&out_pin]);

            // Clock config, measured in kHz to avoid overflows
            let clock_freq = U24F8::from_num(clk_sys_freq() / 1000);
            let ws2812_freq = U24F8::from_num(800);
            let bit_freq = ws2812_freq * CYCLES_PER_BIT;
            cfg.clock_divider = clock_freq / bit_freq;

            // FIFO config
            cfg.fifo_join = FifoJoin::TxOnly;
            cfg.shift_out = ShiftConfig {
                auto_fill: true,
                threshold: 24,
                direction: ShiftDirection::Left,
            };

            sm.set_config(&cfg);
            sm.set_enable(true);

            Self {
                dma: dma.map_into(),
                sm,
            }
        }

        /// Write a buffer of [smart_leds::RGB8] to the ws2812 string
        pub async fn write(&mut self, colors: &[RGB8; N]) {
            // Precompute the word bytes from the colors
            let mut words = [0u32; N];
            for i in 0..N {
                let word = (u32::from(colors[i].g) << 24)
                    | (u32::from(colors[i].r) << 16)
                    | (u32::from(colors[i].b) << 8);
                words[i] = word;
            }

            // DMA transfer
            self.sm.tx().dma_push(self.dma.reborrow(), &words).await;

            Timer::after_micros(55).await;
        }
    }
}

pub mod effects {

    use smart_leds::{brightness, SmartLedsWrite, RGB8};

    use crate::prelude::*;

    pub fn kitt<WS: SmartLedsWrite<Color = RGB8, Error = Error>>(
        ws: &mut WS,
        colors: &mut dyn Iterator<Item = RGB8>,
        data: &mut [RGB8; NUM_LEDS],
    ) {
        let up = 0..MATRIX_WIDTH;
        let down = (0..MATRIX_WIDTH).rev();
        let pause = core::iter::repeat(20).take(100);
        let pause_short = core::iter::repeat(20).take(20);
        let seq = up.chain(pause_short).chain(down).chain(pause);
        for cur in seq {
            data.iter_mut().for_each(|v| {
                *v = brightness(core::iter::once(*v), 210).next().unwrap();
            });
            if cur < MATRIX_WIDTH {
                let c = colors.next().unwrap();

                for y in 0..MATRIX_HEIGHT {
                    set_matrix(cur, y, c, data);
                }
            }
            ws.write(brightness(data.iter().cloned(), 32)).unwrap();
        }
    }
}

pub use smart_leds::RGB8;

pub const NUM_LEDS: usize = 291;
const MATRIX_MAP: [i16; 21 * 19] = [
    291, 291, 291, 291, 291, 291, 291, 291, 0, 1, 2, 3, 4, 5, 6, 7, 291, 291, 291, 291, 291, 291,
    291, 291, 291, 291, 16, 15, 14, 13, 12, 11, 10, 9, 8, 291, 291, 291, 291, 291, 291, 291, 291,
    291, 291, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 291, 291, 291, 291, 291, 291, 291, 291, 37,
    36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 291, 291, 291, 291, 291, 38, 39, 40, 41, 42, 43, 44,
    45, 46, 47, 48, 49, 50, 51, 52, 291, 291, 291, 68, 67, 66, 65, 64, 63, 62, 61, 60, 59, 58, 57,
    56, 55, 54, 53, 291, 291, 291, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84,
    85, 291, 102, 101, 100, 99, 98, 97, 96, 95, 94, 93, 92, 91, 90, 89, 88, 87, 86, 291, 291, 103,
    104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 291, 136, 135,
    134, 133, 132, 131, 130, 129, 128, 127, 126, 125, 124, 123, 122, 121, 120, 291, 291, 137, 138,
    139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 291, 291, 170, 169,
    168, 167, 166, 165, 164, 163, 162, 161, 160, 159, 158, 157, 156, 155, 154, 291, 291, 291, 171,
    172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 291, 291, 204,
    203, 202, 201, 200, 199, 198, 197, 196, 195, 194, 193, 192, 191, 190, 189, 188, 291, 291, 291,
    205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 291, 291,
    237, 236, 235, 234, 233, 232, 231, 230, 229, 228, 227, 226, 225, 224, 223, 222, 291, 291, 291,
    291, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 291, 291, 291,
    291, 291, 291, 291, 263, 262, 261, 260, 259, 258, 257, 256, 255, 254, 253, 291, 291, 291, 291,
    291, 291, 291, 291, 291, 264, 265, 266, 267, 268, 269, 270, 271, 272, 273, 291, 291, 291, 291,
    291, 291, 291, 291, 291, 282, 281, 280, 279, 278, 277, 276, 275, 274, 291, 291, 291, 291, 291,
    291, 291, 291, 291, 291, 291, 283, 284, 285, 286, 287, 288, 289, 290, 291, 291, 291,
];
pub const MATRIX_WIDTH: usize = 19;
pub const MATRIX_HEIGHT: usize = 21;

#[derive(Debug)]
pub enum Error {
    OutOfBounds,
}

pub fn set_matrix(
    x: usize,
    y: usize,
    color: RGB8,
    data: &mut [RGB8; NUM_LEDS],
) -> Result<i16, Error> {
    if x >= MATRIX_WIDTH || y >= MATRIX_HEIGHT {
        return Err(Error::OutOfBounds);
    }
    let addr = x + y * MATRIX_WIDTH;
    let led = MATRIX_MAP.get(addr).ok_or(Error::OutOfBounds)?;
    let rgb = data.get_mut(*led as usize).ok_or(Error::OutOfBounds)?;
    *rgb = color;
    Ok(*led)
}

pub fn get_matrix(x: usize, y: usize, data: &mut [RGB8; NUM_LEDS]) -> Result<(i16, RGB8), Error> {
    if x >= MATRIX_WIDTH || y >= MATRIX_HEIGHT {
        return Err(Error::OutOfBounds);
    }
    let addr = x + y * MATRIX_WIDTH;
    let led = MATRIX_MAP.get(addr).ok_or(Error::OutOfBounds)?;
    Ok((
        *led,
        data.get(*led as usize).cloned().ok_or(Error::OutOfBounds)?,
    ))
}
pub mod prelude {
    pub use super::{
        get_matrix,
        // app, canvas::Canvas, color, color::Rainbow, effects, get_matrix, hal, power_zones,
        // set_matrix, set_matrix_oddr, Console,
        set_matrix,
        Error,
        MATRIX_HEIGHT,
        MATRIX_WIDTH,
        NUM_LEDS,
        RGB8,
    };
}
