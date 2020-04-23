
use std::env;
use std::error::Error;
use std::thread;
use std::time::{Duration};

use rppal::gpio::Gpio;

/// Servo motor has a period of 50Hz (20ms).
/// The position is tied to the pulse width. So a 1.5ms pulse over a 20ms duration is position 0.
/// Opposite the wire input is the 'top' of the servo: 0 degrees.
/// 1500 is position 0. Lower numbers go right. Higher numbers go left.
/// 2500 is -90, 500 is +90. Can be pushed to 2700 and 200.
///
///
/// less than 200 = 1.5 rotations clockwise.
/// Over 3000 = 1.5 (and a bit) rotations anti-clockwise.
/// 6000 - 8000 = 6 oclock shuffle: as a clock, the arm goes 5 - 7 three times.
/// 10000 = 1.5 rotations clockwise
fn feldspar_parachute(_seconds_to_wait: u64, cmds: Vec<u64>) -> Result<(), Box<dyn Error>> {
    const PERIOD_MS: u64 = 20;
    // const PULSE_MIN_US: u64 = 1200;
    // const PULSE_NEUTRAL_US: u64 = 1500;
    // const PULSE_MAX_US: u64 = 1800;
    let pin_num = 23; // BCM pin 23 is physical pin 16
    let mut pin = Gpio::new()?.get(pin_num)?.into_output();

    // Enable software-based PWM with the specified period, and rotate the servo by
    // setting the pulse width to its maximum value.
    for cmd in cmds {
        pin.set_pwm(
            Duration::from_millis(PERIOD_MS),
            Duration::from_micros(cmd),  // 1000 micros = 1 milli.
        )?;

        // Sleep for 500 ms while the servo moves into position.
        thread::sleep(Duration::from_millis(1000));
    }

    Ok(())
}

fn main() {
    let mut args: Vec<String> = env::args().collect();

    args.remove(0);
    let mut parachute_args = Vec::new();
    for a in args.iter() {
        parachute_args.push(a.parse::<u64>().unwrap())
    }
    let _ = feldspar_parachute(7, parachute_args);
}
