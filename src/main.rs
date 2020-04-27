use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime};

use adafruit_gps::gps::{GetGpsData, Gps, open_port};
use adafruit_gps::PMTK::send_pmtk::SendPmtk;
use clap::{App, Arg, SubCommand};
use rppal::gpio::Gpio;

// todo, wait till gps fix is acquired.
fn feldspar_gps(capture_duration: u64, file_name: &str) {
    let port = open_port("/dev/serial0");
    let mut gps = Gps { port };

    gps.pmtk_314_api_set_nmea_output(0, 0, 1, 1, 1, 1, 1);
    let pmtk001 = gps.pmtk_220_set_nmea_updaterate("600");
    dbg!(pmtk001);
    let _file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_name);

    let mut gps_file = OpenOptions::new()
        .append(true)
        .open(file_name) // fails if no file.
        .expect("cannot open file");
    let start_time = SystemTime::now();
    while start_time.elapsed().unwrap() < Duration::from_secs(capture_duration) {
        let gps_values = gps.update();

        gps_file
            .write_all(
                format!(
                    "{:?},{:?},{:?},{:?},{:?},{:?}\n",
                    gps_values.utc,
                    gps_values.latitude,
                    gps_values.longitude,
                    gps_values.speed_kph,
                    gps_values.geoidal_spe,
                    gps_values.altitude
                )
                    .as_bytes(),
            )
            .expect("Failed to write line");
    }
}

fn feldspar_cam(seconds: u64, vid_file: &str) {
    let mili = Duration::from_secs(seconds).as_millis().to_string();
    let _c = Command::new("raspivid")
        .arg("-o")
        .arg(vid_file)
        .arg("-t")
        .arg(mili.as_str())
        .output()
        .expect("Camera failed to open.");
}

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
fn feldspar_parachute(seconds_to_wait: u64, cmds: Vec<[u64; 2]>) {
    const PERIOD_MS: u64 = 20;
    // const PULSE_MIN_US: u64 = 1200;
    // const PULSE_NEUTRAL_US: u64 = 1500;
    // const PULSE_MAX_US: u64 = 1800;
    let pin_num = 23; // BCM pin 23 is physical pin 16
    let mut pin = Gpio::new().unwrap().get(pin_num).unwrap().into_output();

    thread::sleep(Duration::from_secs(seconds_to_wait));
    // Enable software-based PWM with the specified period, and rotate the servo by
    // setting the pulse width to its maximum value.
    for cmd_pair in cmds {
        let cmd = cmd_pair[0];
        let wait = cmd_pair[1];
        pin.set_pwm(
            Duration::from_millis(PERIOD_MS),
            Duration::from_micros(cmd),  // 1000 micros = 1 milli.
        ).unwrap();
        // Sleep for 500 ms while the servo moves into position.
        thread::sleep(Duration::from_millis(wait));
    }
}

fn main() {
    let args = App::new("Feldspar Launch Protocol")
        .version("1.0")
        .author("Matt")
        .arg(Arg::with_name("Recording duration")
            .short("d")
            .long("duration")
            .value_name("RECORDING DURATION")
            .help("Sets the amount of time you want the instruments to record.")
            .takes_value(true))
        .arg(Arg::with_name("Parachute Deployment Max Time")
            .short("p")
            .long("parachute")
            .value_name("Parachute Delay")
            .help("Sets the maximum time delay after launch when the parachute will deploy")
            .takes_value(true))
        .arg(Arg::with_name("Flight Name")
            .short("n")
            .long("name")
            .value_name("Flight Name")
            .help("The name of the launch: 3-0 or 4-5")
            .takes_value(true))
        .get_matches();


    let recording_duration = args.value_of("Recording duration")
        .parse::<u64>()
        .expect("Please enter a valid integer for launch duration");

    let deploy_delay = args.value_of("Parachute Deployment Max Time").parse::<u64>().expect("Enter a valid integer for deployment time (seconds)");

    let feldspar_number = args.value_of("Flight Name");

    let vid_name = format!("./feldspar{}_vid.h264", feldspar_number);
    let gps_file_name = format!("./feldspar{}_gps.txt", feldspar_number);

    if Path::new(vid_name.as_str()).exists() || Path::new(gps_file_name.as_str()).exists() {
        panic!("Change feldspar launch type, there is a file name conflict.")
    }

    println!("Standby for feldspar launch {}...", feldspar_number);
    println!("Total rocket flight time is {}", launch_duration);
    println!("Parachute deploy in {} seconds after launch", deploy_delay);

    println!("Init servo");
    feldspar_parachute(0, vec![[500, 500]]);
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

    for i in 0..deploy_delay {
        println!("-{}", i);
        thread::sleep(Duration::from_secs(1));
    }
    println!("Deploy!");
    feldspar_parachute(0, vec![[2500, 1000], [500, 500]]);

    cam_thread.join().unwrap();
    gps_thread.join().unwrap();
}
