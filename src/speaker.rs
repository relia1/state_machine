use microbit::{
    hal::{gpio, gpio::p0::Parts, pwm},
    pac,
};

/// Speaker can have one of two states On/Off
/// While On play a square wave through the speaker on loop
/// using dma
#[derive(Debug, Clone, Copy)]
pub enum Speaker {
    On,
    Off,
}

impl Speaker {
    /// Speakers default state is off
    pub fn new() -> Self {
        Self::Off
    }

    /// Turn on the speaker looping our sequences
    pub fn on(&self) {
        // Board take will return none if it has already been called
        // which it has in main.rs, so until I sort out how to do this better
        // it is wrapped in an unsafe
        unsafe {
            let p = pac::Peripherals::steal();
            let my_pins = Parts::new(p.P0);
            let speaker_pin = my_pins
                .p0_00
                .degrade()
                .into_push_pull_output(gpio::Level::Low);

            // https://github.com/pdx-cs-rust-embedded/mb2-audio-experiments/blob/hw-pwm/src/main.rs
            // https://github.com/pdx-cs-rust-embedded/hello-audio/blob/main/src/main.rs
            // referenced as examples
            let speaker = pwm::Pwm::new(p.PWM0);
            speaker
                .set_output_pin(pwm::Channel::C0, speaker_pin)
                .set_prescaler(pwm::Prescaler::Div1)
                .set_counter_mode(pwm::CounterMode::Up)
                .set_load_mode(pwm::LoadMode::Common)
                .set_step_mode(pwm::StepMode::Auto)
                .set_max_duty(128)
                .enable_channel(pwm::Channel::C0)
                .enable_group(pwm::Group::G0)
                .loop_inf()
                .enable();

            static mut SQUARE_WAVE0: [u16; 64] = [0x8000; 64];
            static mut SQUARE_WAVE1: [u16; 64] = [0x0000; 64];

            // Start the square wave
            let _pwm_seq = speaker
                .load(Some(&SQUARE_WAVE0), Some(&SQUARE_WAVE1), true)
                .unwrap();
        }
    }

    /// Turns off the speaker (stable state)
    pub fn off(&self) {
        unsafe {
            let p = pac::Peripherals::steal();
            let speaker = pwm::Pwm::new(p.PWM0);
            speaker.disable();
        }
    }
}
