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
                    .side_set 2 opt
                    set pindirs, 0b11               
                    .wrap_target
                    frame1:
                        set x, 29      side 0b00
                        nop            side 0b01
                        nop            side 0b00
                    data1:
                        in pins, 1     side 0b01
                        jmp x-- data1  side 0b00
                    frame2:
                        in pins, 1     side 0b01
                        set x, 29      side 0b10
                        nop            side 0b11
                        nop            side 0b10
                    data2:
                        nop            side 0b11
                        jmp x-- data2  side 0b10
                        nop            side 0b11
                    .wrap
                "#
        );
        let prg = common.load_program(&prg.program);
        Self { prg }
    }
}

/// Pio backed ws2812 driver
/// Const N is the number of ws2812 leds attached to this pin
pub struct PioI2S<'d, P: Instance, const S: usize> {
    dma: PeripheralRef<'d, AnyChannel>,
    sm: StateMachine<'d, P, S>,
}

impl<'d, P: Instance, const S: usize> PioI2S<'d, P, S> {
    /// Configure a pio state machine to use the loaded ws2812 program.
    pub fn new(
        pio: &mut Common<'d, P>,
        mut sm: StateMachine<'d, P, S>,
        dma: impl Peripheral<P = impl Channel> + 'd,
        pin: impl PioPin,
        pinwc: impl PioPin,
        pindata: impl PioPin,
        program: &PioI2SProgram<'d, P>,
    ) -> Self {
        into_ref!(dma);

        // Setup sm0
        let mut cfg = Config::default();

        // Pin config
        let out_pin = pio.make_pio_pin(pin);
        let out_pinwc = pio.make_pio_pin(pinwc);
        let mut in_pindata = pio.make_pio_pin(pindata);
        in_pindata.set_input_sync_bypass(true);
        // cfg.set_out_pins(&[&out_pin, &out_pinwc]);
        cfg.set_set_pins(&[&out_pin, &out_pinwc]);
        cfg.set_in_pins(&[&in_pindata]);

        cfg.use_program(&program.prg, &[&out_pin, &out_pinwc]);

        // Clock config, measured in kHz to avoid overflows
        let clock_freq = U24F8::from_num(clk_sys_freq() / 2000);
        let i2s_freq = U24F8::from_num(1411);
        let bit_freq = i2s_freq * 2;
        cfg.clock_divider = clock_freq / bit_freq;

        // FIFO config
        cfg.fifo_join = FifoJoin::RxOnly;
        // cfg.shift_out = ShiftConfig {
        //     auto_fill: true,
        //     threshold: 24,
        //     direction: ShiftDirection::Left,
        // };
        cfg.shift_in = ShiftConfig {
            auto_fill: true,
            threshold: 31,
            direction: ShiftDirection::Left,
        };
        sm.set_config(&cfg);
        sm.set_enable(true);

        Self {
            dma: dma.map_into(),
            sm,
        }
    }
    pub async fn read<const N: usize>(&mut self, samples: &mut [u32; N]) {
        self.sm.rx().dma_pull(self.dma.reborrow(), samples).await;

        // Timer::after_micros(55).await;
    }
}
