// State machine for falling microbit v2
use rtt_target::rprintln;

use crate::leds::Leds;
use crate::speaker::Speaker;
use micromath::F32Ext;

#[derive(Debug, Clone, Copy)]
pub enum MB2 {
    Stable(FreeFallState),
    Falling(FreeFallState),
}

impl MB2 {
    pub fn new() -> Self {
        Self::Stable(FreeFallState::Stable(Leds::new(), Speaker::new()))
    }

    pub fn on_entry(&mut self) {
        match self {
            Self::Stable(..) => {
                #[cfg(debug_assertions)]
                rprintln!(
                    "Entering stable state!\nTurning off the speaker and showing default display"
                );

                self::Leds::CenterLED.default_display();
                self::Speaker::On.off();
            }
            Self::Falling(..) => {
                #[cfg(debug_assertions)]
                rprintln!("Entering falling state!\n");

                self::Leds::ExclamationMark.falling_display();
                self::Speaker::Off.on();
            }
        };
    }

    pub fn next(&mut self) -> Self {
        match self {
            Self::Stable { .. } => {
                *self = Self::Falling(FreeFallState::Falling(Leds::ExclamationMark, Speaker::On));
                self.on_entry();
                *self
            }
            Self::Falling { .. } => {
                *self = Self::Stable(FreeFallState::Stable(Leds::CenterLED, Speaker::Off));
                self.on_entry();
                *self
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FreeFallState {
    Falling(Leds, Speaker),
    Stable(Leds, Speaker),
}

pub struct BoardAccel {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    sample_size: i32,
    state: MB2,
}

impl BoardAccel {
    pub fn new() -> BoardAccel {
        Self {
            x: 0,
            y: 0,
            z: 0,
            sample_size: 0,
            state: MB2::new(),
        }
    }

    pub fn add_to_total(&mut self, x: i32, y: i32, z: i32) {
        self.x = self.x + x;
        self.y = self.y + y;
        self.z = self.z + z;
        self.sample_size = self.sample_size + 1;
    }

    pub fn add_tuple_to_total(&mut self, accel_tuple: (i32, i32, i32)) {
        let (x, y, z): (i32, i32, i32) = accel_tuple;
        self.add_to_total(x, y, z);
    }

    pub fn average_over_sample(&mut self) -> (i32, i32, i32) {
        let (x, y, z) = (
            self.x / self.sample_size,
            self.y / self.sample_size,
            self.z / self.sample_size,
        );
        self.reset();
        (x, y, z)
    }

    pub fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
        self.z = 0;
        self.sample_size = 0;
    }

    pub fn microbit_is_falling(&mut self, x: f32, y: f32, z: f32) {
        let combined_strength = x.powf(2.0) + y.powf(2.0) + z.powf(2.0);
        let result = combined_strength.sqrt() / 1000.0;
        self.state = match self.state {
            MB2::Falling(..) => {
                if result < 0.55 {
                    self.state
                } else {
                    self.state.next()
                }
            }
            MB2::Stable(..) => {
                if result < 0.5 {
                    self.state.next()
                } else {
                    self.state
                }
            }
        }
    }
}
