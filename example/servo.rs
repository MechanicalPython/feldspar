use rppal::gpio::Gpio;
use std::env;
use std::thread;
use std::time::{Duration, SystemTime};

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    let pin_num = 23; // BCM pin 23 is physical pin 16
    let mut pin = Gpio::new().unwrap().get(pin_num).unwrap().into_output();

    // Enable software-based PWM with the specified period, and rotate the servo by
    // setting the pulse width to its maximum value.
    for arg in args {
        pin.set_pwm(
            Duration::from_millis(PERIOD_MS),
            Duration::from_micros(arg),  // 1000 micros = 1 milli.
        ).unwrap();
        // Sleep for 500 ms while the servo moves into position.
        thread::sleep(Duration::from_millis(1000));
    }
}