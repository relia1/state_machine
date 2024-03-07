#![no_main]
#![no_std]

mod leds;
mod speaker;
mod state;

use cortex_m::asm;
use cortex_m_rt::entry;
use critical_section_lock_mut::LockMut;
use leds::DISPLAY;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use state::*;

use microbit::{
    display::nonblocking::Display,
    hal::{delay::Delay, gpiote::*, prelude::*, twim, Timer},
    pac::{self as pac, interrupt, twim0::frequency::FREQUENCY_A},
    Board,
};

use lsm303agr::{AccelScale, Interrupt, Lsm303agr};

/// Global mutex used for our accelerometer interrupt
pub static GPIOTE: LockMut<Gpiote> = LockMut::new();

#[entry]
fn main() -> ! {
    rtt_init_print!();

    // Grab board
    let mut board = match Board::take() {
        Some(res) => res,
        None => {
            panic!("There was a problem taking the board\n");
        }
    };

    let mut timer = Timer::new(board.TIMER0);
    // There is an issue regarding interrupts/combined interrupt line
    // that makes it so we need to do a work around of a small delay
    // then a dummy read to i2c
    let mut delay = Delay::new(board.SYST);
    delay.delay_ms(1000u16);

    let display = Display::new(board.TIMER1, board.display_pins);
    DISPLAY.init(display);

    // Initializing program data struct
    let mut board_accel = BoardAccel::new();
    let mut i2c = twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);
    // Dummy read to clear line from being held down
    let mut buf = [0; 4];
    let i2c_read = i2c.read(0x70, &mut buf);

    match i2c_read {
        Ok(()) => {
            rprintln!("i2c read was successful\n");
        }
        Err(err) => {
            rprintln!("i2c read result: {:?}\n", i2c_read);
            rprintln!("Something went wrong: {:?}\n", err);
            i2c.disable();
            delay.delay_ms(1000u16);
            i2c.enable();
            delay.delay_ms(1000u16);
            match i2c.read(0x19, &mut buf) {
                Ok(()) => {
                    rprintln!("i2c read was successful on the second try\n");
                }
                Err(err2) => {
                    panic!("Something went wrong: {:?}\n", err2);
                }
            };
        }
    };

    let mut sensor = Lsm303agr::new_with_i2c(i2c);

    match sensor.init() {
        Ok(()) => {
            rprintln!("success\n");
        }
        Err(err) => {
            rprintln!("Something went wrong: {:?}\n", err);
        }
    };

    let res = sensor.set_accel_mode_and_odr(
        &mut timer,
        lsm303agr::AccelMode::LowPower,
        lsm303agr::AccelOutputDataRate::Hz10,
    );
    match res {
        Ok(()) => {
            rprintln!("Setting accel mode and odr\n");
        }
        Err(err) => {
            panic!("Error setting accel mode and odr: {:?}\n", err);
        }
    };

    // Setup the accelerometer so that the gravity scale is +-2G
    // Enable dataready interrupt
    sensor.set_accel_scale(AccelScale::G2).unwrap();
    sensor.acc_enable_interrupt(Interrupt::DataReady1).unwrap();
    let gpiote = Gpiote::new(board.GPIOTE);
    let channel = gpiote.channel0();
    channel
        .input_pin(&board.pins.p0_25.degrade().into_pullup_input())
        .enable_interrupt()
        .hi_to_lo();
    channel.reset_events();
    GPIOTE.init(gpiote);

    unsafe {
        board.NVIC.set_priority(pac::Interrupt::GPIOTE, 2);
        board.NVIC.set_priority(pac::Interrupt::TIMER1, 6);
        pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        pac::NVIC::unmask(pac::Interrupt::TIMER1);
    }

    pac::NVIC::unpend(pac::Interrupt::GPIOTE);
    pac::NVIC::unpend(pac::Interrupt::TIMER1);

    // Counter used for entrance into the first if statement meaning we
    // have enough samples to make our calculation
    let mut counter: u8 = 0;
    loop {
        if counter >= 5 {
            let (x, y, z) = board_accel.average_over_sample();
            board_accel.microbit_is_falling(x as f32, y as f32, z as f32);
            counter = 0;
            #[cfg(debug_assertions)]
            rprintln!("x: {}, y: {}, z: {}\n", x, y, z);
        }

        if sensor.accel_status().unwrap().xyz_new_data() {
            board_accel.add_tuple_to_total(sensor.acceleration().unwrap().xyz_mg());
            counter += 1;
        }
        asm::wfi();
    }
}

#[interrupt]
fn GPIOTE() {
    GPIOTE.with_lock(|gpiote| {
        gpiote.channel0().reset_events();
    });
}

#[interrupt]
fn TIMER1() {
    DISPLAY.with_lock(|display| {
        display.handle_display_event();
    });
}
