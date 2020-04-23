
use std::env;
use std::error::Error;
use std::thread;
use std::time::{Duration};

use rppal::gpio::Gpio;


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
            Duration::from_micros(cmd),
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
