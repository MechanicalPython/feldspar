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

    let launch_duration: &str = args
        .get(1)
        .expect("Please enter an instrument recording time (seconds)");
    let launch_duration = launch_duration
        .parse::<u64>()
        .expect("Please enter a valid integer for launch duration");
    let feldspar_number = args.get(2).expect("Please enter feldspar launch number.");

    let vid_name = format!("./feldspar{}_vid.h264", feldspar_number);
    let gps_file_name = format!("./feldspar{}_gps.txt", feldspar_number);

    if Path::new(vid_name.as_str()).exists() || Path::new(gps_file_name.as_str()).exists() {
        panic!("Change feldspar launch type, there is a file name conflict.")
    }

    println!("Standby for feldspar launch {}...", feldspar_number);
    thread::sleep(Duration::from_secs(2));
    println!("Instrument recording time is {}", launch_duration);

    println!("Press enter to begin launch countdown.");
    let mut s = String::new();
    let _stdin = io::stdin().read_line(&mut s).unwrap();

    let gps_thread = thread::spawn(move || {
        println!("Starting gps...");
        feldspar_gps(launch_duration + 10, gps_file_name.as_str())
    });

    let cam_thread = thread::spawn(move || {
        println!("Starting camera...");
        feldspar_cam(launch_duration + 10, vid_name.as_str());
    });

    for i in (1..11).rev() {
        println!("{}", i);
        thread::sleep(Duration::from_secs(1));
    }
    println!("Launch!");
    for i in (0..launch_duration - 10).rev() {
        println!("{}", i);
    }

    feldspar_parachute(7, parachute_args);

    cam_thread.join().unwrap();
    gps_thread.join().unwrap();
}
