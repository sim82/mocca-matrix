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

/// This struct represents a ws2812 program loaded into pio instruction memory.
pub struct PioI2SProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> PioI2SProgram<'a, PIO> {
    /// Load the ws2812 program into the given pio
    pub fn new2(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio_proc::pio_asm!(
            r#"
                    .side_set 1
                    set pindirs, 1               side 0
                    .wrap_target
                        set x, 32            side 1
                        set pins 0b10            side 1
                    bitloop:
                        nop                   side 0
                        nop                   side 1
                        jmp x-- bitloop        side 0
                        set x, 32            side 1
                        set pins 0            side 1
                    bitloop2:
                        nop                   side 0
                        nop                   side 1
                        jmp x-- bitloop2        side 0
                    .wrap
                "#
        );
        let prg = common.load_program(&prg.program);
        Self { prg }
    }
}

/// Pio backed ws2812 driver
/// Const N is the number of ws2812 leds attached to this pin
pub struct PioI2S<'d, P: Instance, const S: usize, const N: usize> {
    dma: PeripheralRef<'d, AnyChannel>,
    sm: StateMachine<'d, P, S>,
}

impl<'d, P: Instance, const S: usize, const N: usize> PioI2S<'d, P, S, N> {
    /// Configure a pio state machine to use the loaded ws2812 program.
    pub fn new(
        pio: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: impl Peripheral<P = impl Channel> + 'd,
        pin: impl PioPin,
        pinwc: impl PioPin,
        program: &PioI2SProgram<'d, P>,
    ) -> Self {
        into_ref!(dma);

        // Setup sm0
        let mut cfg = Config::default();

        // Pin config
        let out_pin = pio.make_pio_pin(pin);
        let out_pinwc = pio.make_pio_pin(pinwc);
        cfg.set_out_pins(&[&out_pin, &out_pinwc]);
        cfg.set_set_pins(&[&out_pin, &out_pinwc]);

        cfg.use_program(&program.prg, &[&out_pin]);

        // Clock config, measured in kHz to avoid overflows
        let clock_freq = U24F8::from_num(clk_sys_freq() / 1000);
        let i2s_freq = U24F8::from_num(1000);
        let bit_freq = i2s_freq * 2;
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
