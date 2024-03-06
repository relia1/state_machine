#![no_main]
#![no_std]

use cortex_m::asm;
use cortex_m_rt::entry;
use critical_section_lock_mut::LockMut;
use microbit::*;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
mod leds;
mod speaker;
mod state;
use crate::leds::DISPLAY;
use state::*;

// use critical_section_lock_mut::LockMut;
use microbit::pac::{self as pac, interrupt, twim0::frequency::FREQUENCY_A};
use microbit::{
    display::nonblocking::Display,
    hal::{delay::Delay, gpiote::*, prelude::*, twim, Timer},
};

use lsm303agr::{AccelScale, Interrupt, Lsm303agr};

pub static GPIOTE: LockMut<Gpiote> = LockMut::new();

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut delay = Delay::new(board.SYST);
    delay.delay_ms(1000u16);

    let display = Display::new(board.TIMER1, board.display_pins);
    DISPLAY.init(display);

    let mut board_accel = BoardAccel::new();
    let mut i2c = twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);
    let mut buf: [u8; 4] = [0, 0, 0, 0];
    let _ = i2c.read(0x70, &mut buf);

    let mut sensor = Lsm303agr::new_with_i2c(i2c);

    sensor.init().unwrap();
    sensor
        .set_accel_mode_and_odr(
            &mut timer,
            lsm303agr::AccelMode::LowPower,
            lsm303agr::AccelOutputDataRate::Hz100,
        )
        .unwrap();

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

    sensor.acceleration().unwrap().xyz_mg();
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
