use critical_section_lock_mut::LockMut;
use microbit::display::nonblocking::{BitImage, Display};
use microbit::pac::TIMER1;

pub static DISPLAY: LockMut<Display<TIMER1>> = LockMut::new();

#[derive(Debug, Clone, Copy)]
pub enum Leds {
    CenterLED,
    ExclamationMark,
}

impl Leds {
    pub fn new() -> Self {
        let display_leds = Self::CenterLED;
        display_leds.default_display();
        display_leds
    }

    pub fn falling_display(&self) {
        let image: [[u8; 5]; 5] = [
            [0, 0, 1, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0],
        ];
        DISPLAY.with_lock(|display| display.show(&BitImage::new(&image)));
    }

    pub fn default_display(&self) {
        let image: [[u8; 5]; 5] = [
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ];
        DISPLAY.with_lock(|display| display.show(&BitImage::new(&image)));
    }
}
