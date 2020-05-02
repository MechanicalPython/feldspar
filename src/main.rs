use std::fs::OpenOptions;
use std::io::{self, Write, stdout};
use std::path::Path;
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, SystemTime};

use adafruit_gps::gps::{GetGpsData, Gps, open_port};
use adafruit_gps::PMTK::send_pmtk::{set_baud_rate};
use clap::{App, Arg};
use rppal::gpio::Gpio;

fn feldspar_gps(capture_duration: u64, file_name: &str) -> f32 {
    let port = open_port("/dev/serial0", 57600);
    let mut gps = Gps { port , satellite_data: false, naviagtion_data: true };

    let _file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_name);

    let mut gps_file = OpenOptions::new()
        .append(true)
        .open(file_name) // fails if no file.
        .expect("cannot open file");

    let mut max_alt: f32 = 0.0;
    let start_time = SystemTime::now();
    while start_time.elapsed().unwrap() < Duration::from_secs(capture_duration) {
        let gps_values = gps.update();
        if gps_values.altitude.unwrap_or(0.0) > max_alt {
            max_alt = gps_values.altitude.unwrap()
        }
        gps_file
            .write_all(
                format!(
                    "{:?},{:?},{:?},{:?},{:?},{:?},{:?}\n",
                    gps_values.utc,
                    gps_values.latitude,
                    gps_values.longitude,
                    gps_values.sats_used,
                    gps_values.speed_kph,
                    gps_values.geoidal_spe,
                    gps_values.altitude
                )
                    .as_bytes(),
            )
            .expect("Failed to write line");
    }
    return max_alt;
}

fn gps_checker() {
    let _ = set_baud_rate("57600", "/dev/serial0");

    let port = open_port("/dev/serial0", 57600);
    let mut gps = Gps { port, satellite_data: false, naviagtion_data: true };
    gps.init("100");

    let stdout = stdout();
    let mut handle = stdout.lock();

    let mut count = 0;
    loop {
        let gps_values = gps.update();
        handle.write_all(format!("\rGPS satellites found: {}", gps_values.sats_used).as_bytes()).unwrap();
        handle.flush().unwrap();
        thread::sleep(Duration::from_millis(100));
        count += 1;
        if count > 5 {
            if gps_values.sats_used > 6 {
                return ()
            }
            println!("\nPress enter to continue the search. Press c to cancel search and continue.");
            let mut s = String::new();
            io::stdin().read_line(&mut s).unwrap();
            if s.trim() == "c".to_string() {
                return ()
            } else {
                count = 0;
            }
        }
    }
}

fn feldspar_cam(vid_file: &str) -> Output {
    let c = Command::new("raspivid")
        .arg("-o")
        .arg(vid_file)
        .arg("-t")
        .arg("0")
        .output()
        .expect("Camera failed to open.");
    return c
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
    let pin_num = 23; // BCM pin 23 is physical pin 16
    let mut pin = Gpio::new().unwrap().get(pin_num).unwrap().into_output();

    for i in (1..seconds_to_wait+1).rev() {
        println!("Deploy in {}", i);
        thread::sleep(Duration::from_secs(1));
    }
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


    let recording_duration = args.value_of("Recording duration").unwrap()
        .parse::<u64>()
        .expect("Please enter a valid integer for launch duration");

    let deploy_delay = args.value_of("Parachute Deployment Max Time").unwrap().parse::<u64>().expect("Enter a valid integer for deployment time (seconds)");

    let feldspar_number = args.value_of("Flight Name").unwrap();

    let vid_name = format!("./feldspar{}_vid.h264", feldspar_number);
    let gps_file_name = format!("./feldspar{}_gps.txt", feldspar_number);

    if Path::new(vid_name.as_str()).exists() || Path::new(gps_file_name.as_str()).exists() {
        panic!("Change feldspar launch type, there is a file name conflict.")
    }

    println!("Standby for feldspar launch {}...", feldspar_number);
    println!("Total rocket flight time is {}", recording_duration);
    println!("Parachute deploy in {} seconds after launch", deploy_delay);

    let cam_thread = thread::spawn(move || {
        println!("Starting camera...");
        feldspar_cam(vid_name.as_str());
    });


    println!("Initialise servo");
    feldspar_parachute(0, vec![[2500, 500]]);


    println!("Check Gps");
    gps_checker();

    println!("Press enter to begin launch countdown.");
    let mut s = String::new();
    let _stdin = io::stdin().read_line(&mut s).unwrap();


    let gps_thread = thread::spawn(move || {
        println!("Starting gps...");
        let max_alt = feldspar_gps(recording_duration + 10, gps_file_name.as_str());
        println!("Maximum altitude: {}", max_alt);
    });


    for i in (1..11).rev() {
        println!("{}", i);
        thread::sleep(Duration::from_secs(1));
    }
    println!("Launch!");
    let parachute_thread = thread::spawn(move || {
        feldspar_parachute(deploy_delay, vec![[500, 1000], [2500, 500]]);
        println!("Deployed!");
    });

    for i in 1..recording_duration {
        println!("-{}",i);
        thread::sleep(Duration::from_secs(1));
    }

    gps_thread.join().unwrap();
    parachute_thread.join().unwrap();
}
