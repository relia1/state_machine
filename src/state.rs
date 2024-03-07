// State machine for falling microbit v2
use rtt_target::rprintln;

use crate::leds::Leds;
use crate::speaker::Speaker;
use micromath::F32Ext;

/// The microbit will have one of two states
/// Stable which is the default state
/// Falling which is the state in which the acceleration detected by the
/// lsm303agr eclipses 50% of the acceleration of gravity
#[derive(Debug, Clone, Copy)]
pub enum MB2 {
    Stable(FreeFallState),
    Falling(FreeFallState),
}

impl MB2 {
    /// Creates with default state of Stable
    pub fn new() -> Self {
        Self::Stable(FreeFallState::Stable(Leds::new(), Speaker::new()))
    }

    /// Upon entering the state make sure display and speaker are updated
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

    /// Based on the information we alrady have about the current state setup
    /// the transition to what the next state is
    /// Default (AKA Stable)->Falling
    /// Falling->Stable
    /// Stable->Falling
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

/// The two states that the board can be in
/// Falling which has specific LED/speaker behavior
/// Stable which has specific LED/speaker behavior
#[derive(Debug, Clone, Copy)]
pub enum FreeFallState {
    Falling(Leds, Speaker),
    Stable(Leds, Speaker),
}

/// Data struct that the main program uses for acceleration calculations
pub struct BoardAccel {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    sample_size: i32,
    state: MB2,
}

impl BoardAccel {
    /// Create new BoardAccel with default values
    pub fn new() -> BoardAccel {
        Self {
            x: 0,
            y: 0,
            z: 0,
            sample_size: 0,
            state: MB2::new(),
        }
    }

    /// Add the x,y,z acceleration values to the total and increment the
    /// sample size
    pub fn add_to_total(&mut self, x: i32, y: i32, z: i32) {
        self.x += x;
        self.y += y;
        self.z += z;
        self.sample_size += 1;
    }

    /// Provides an additional option for adding to the total
    /// Instead of 3 args allow a tuple to be sent
    pub fn add_tuple_to_total(&mut self, accel_tuple: (i32, i32, i32)) {
        let (x, y, z): (i32, i32, i32) = accel_tuple;
        self.add_to_total(x, y, z);
    }

    /// Calculate the average over a number of samples then reset self values
    /// back to default
    /// Returns the calculated x,y,z as a tuple
    pub fn average_over_sample(&mut self) -> (i32, i32, i32) {
        let (x, y, z) = (
            self.x / self.sample_size,
            self.y / self.sample_size,
            self.z / self.sample_size,
        );
        self.reset();
        (x, y, z)
    }

    /// Reset self values back to their defaults
    pub fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
        self.z = 0;
        self.sample_size = 0;
    }

    /// Given the provided x, y, z values determine if the microbit is falling
    /// The cartesian magnitude of the acceleration can be calculated like so
    /// a = sqrt((x^2)+(y^2)+(z^2))
    ///
    /// We are using a threshold to determine if the microbit is falling of 50%
    /// that of gravity. The threshold for returning to Stable is set to 55% to
    /// prevent any possible minor oscillating around 50% rapidly triggering it
    /// on and off
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
