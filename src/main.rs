use std::env;
use std::fs::OpenOptions;
use std::io::{self, stdout, Write};
use std::path::Path;
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, SystemTime};

use adafruit_gps::gps::{GetGpsData, Gps, open_port};
use adafruit_gps::PMTK::send_pmtk::set_baud_rate;
use rppal::gpio::{Gpio, OutputPin};

fn feldspar_gps(capture_duration: u64, file_name: &str) -> f32 {
    let port = open_port("/dev/serial0", 57600);
    let mut gps = Gps { port, satellite_data: false, naviagtion_data: true };

    let _file = OpenOptions::new().write(true).create_new(true).open(file_name);

    let mut gps_file = OpenOptions::new().append(true).open(file_name).expect("cannot open file");

    let mut max_alt: f32 = 0.0;

    let mut cont = true;
    while cont {
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
                return ();
            }
            println!("\nPress enter to continue the search. Press c to cancel search and continue.");
            let mut s = String::new();
            io::stdin().read_line(&mut s).unwrap();
            if s.trim() == "c".to_string() {
                return ();
            } else {
                count = 0;
            }
        }
    }
}

/// Just keeps on recording.
fn feldspar_cam(vid_file: &str) -> Output {
    let c = Command::new("raspivid")
        .arg("-o")
        .arg(vid_file)
        .arg("-t")
        .arg("0")
        .output()
        .expect("Camera failed to open.");
    return c;
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

struct Parachute {
    pin_number: u8,
}

impl Parachute {
    pub fn init_servo(&mut self) {
        self.pin = Gpio::new().unwrap().get(self.pin).unwrap().into_output();
    }

    pub fn move_servo(&mut self, pos: u64, delay: u64) {
        self.pin.set_pwm(
            Duration::from_millis(20),
            Duration::from_micros(pos),  // 1000 micros = 1 milli.
        ).unwrap();
        // Sleep for 500 ms while the servo moves into position.
        thread::sleep(Duration::from_millis(delay));
    }
}

fn main() {
    let args: Vec<str> = env::args().collect();
    let feldspar_number = args.get(1).unwrap();

    let vid_name = format!("./feldspar{}_vid.h264", feldspar_number);
    let gps_file_name = format!("./feldspar{}_gps.txt", feldspar_number);

    if Path::new(vid_name.as_str()).exists() || Path::new(gps_file_name.as_str()).exists() {
        panic!("Change feldspar launch type, there is a file name conflict.")
    }
    println!("Standby for feldspar launch {}...", feldspar_number);

    let parachute_thread = thread::spawn(|| {
        let mut parachute = Parachute { pin_number: 23 };
        parachute.init_servo();
        parachute.move_servo(2500, 500);
        println!("Parachute OK")
    });

    let camera_thread = thread::spawn(|| {
        // Will record continously.
        let r = feldspar_cam(vid_name.as_str());
        if r.status.code().unwrap_or_default() != 0 {
            println!("Camera error. Check connection")
        };
    });


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

    let cam_thread = thread::spawn(move || {
        println!("Starting camera...");
        feldspar_cam(vid_name.as_str());
    });

    for i in (1..11).rev() {
        println!("{}", i);
        thread::sleep(Duration::from_secs(1));
    }
    println!("Launch!");
    let parachute_thread = thread::spawn(move || {
        feldspar_parachute(vec![[500, 1000], [2500, 500]]);
        println!("Deployed!");
    });

    for i in 1..recording_duration {
        println!("-{}", i);
        thread::sleep(Duration::from_secs(1));
    }

    gps_thread.join().unwrap();
}
