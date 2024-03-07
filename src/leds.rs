use critical_section_lock_mut::LockMut;
use microbit::display::nonblocking::{BitImage, Display};
use microbit::pac::TIMER1;

/// Global LED display mutex
pub static DISPLAY: LockMut<Display<TIMER1>> = LockMut::new();

/// The LEDs can have 1 of 2 states
/// CenterLED is the default state and also the state it is in when the
/// accelerometer is stable
/// ExclamationMark is the state for when the accelerometer has detected
/// it is falling
#[derive(Debug, Clone, Copy)]
pub enum Leds {
    CenterLED,
    ExclamationMark,
}

impl Leds {
    /// Create LEDs with the default state
    pub fn new() -> Self {
        let display_leds = Self::CenterLED;
        display_leds.default_display();
        display_leds
    }

    /// Image to be displayed when the microbit is falling
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

    /// Default image to be displayed and the image shown when the microbit is
    /// stable
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
