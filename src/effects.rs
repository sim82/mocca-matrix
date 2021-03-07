use smart_leds::{brightness, SmartLedsWrite, RGB8};

use crate::prelude::*;

pub fn kitt<WS: SmartLedsWrite<Color = RGB8, Error = hal::spi::Error>>(
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
