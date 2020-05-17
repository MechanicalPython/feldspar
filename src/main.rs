use std::fs::OpenOptions;
use std::io::{self, stdout, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration};

use adafruit_gps::gps::{Gps, GpsSentence, open_port};
use adafruit_gps::PMTK::send_pmtk::{Pmtk001Ack, set_baud_rate};
use clap::{App, Arg};
use rppal::gpio::Gpio;

fn feldspar_gps(file_name: &str, rx: Receiver<bool>) -> f32 {
    let port = open_port("/dev/serial0", 57600);
    let mut gps = Gps { port };

    let mut gps_file = OpenOptions::new()
        .create_new(true)
        .append(true)
        .open(file_name)
        .expect("File already exits.");

    let mut max_alt: f32 = 0.0;
    loop {
        let mut utc: f64 = 0.0;
        let mut latitude = None;
        let mut longitude = None;
        let mut altitude = None;
        let mut vdop = None;
        let mut hdop = None;
        let mut pdop = None;

        match gps.update() {
            GpsSentence::GGA(sentence) => {
                utc = sentence.utc;
                latitude = sentence.lat;
                longitude = sentence.long;
                altitude = sentence.msl_alt;
            }
            GpsSentence::GSA(sentence) => {
                vdop = sentence.vdop;
                hdop = sentence.hdop;
                pdop = sentence.pdop;
            }
            _ => {}
        }

        if altitude.unwrap_or(0.0) > max_alt {
            max_alt = altitude.unwrap()
        }

        if latitude.is_some() && longitude.is_some() && altitude.is_some() && vdop.is_some() &&
            hdop.is_some() && pdop.is_some() {
            gps_file.write_all(format!("{},{},{},{},{},{},{}\n",
                                       utc, latitude.unwrap(), longitude.unwrap(), altitude.unwrap(),
                                       vdop.unwrap(), hdop.unwrap(), pdop.unwrap())
                .as_bytes())
                .unwrap_or(());
        } else {
            gps_file.write_all(format!("{},None,None,None,None,None,None\n", utc)
                .as_bytes())
                .unwrap_or(());
        }

        if rx.try_recv().unwrap_or(false) {
            break;
        }
    }
    return max_alt;
}

fn gps_checker() {
    let port = open_port("/dev/serial0", 57600);
    let mut gps = Gps { port };

    match gps.update() {
        GpsSentence::NoConnection => {
            println!("GPS not connected");
            ()
        }
        GpsSentence::InvalidBytes => {
            println!("GPS baud rate not correct");
            set_baud_rate("57600", "/dev/serial0");
        }
        _ => {}
    };

    let nmea_output = gps.pmtk_314_api_set_nmea_output(0, 0, 0, 1, 0, 0, 1);
    println!("GGA output only: {:?}", nmea_output);

    let valid_hz = ["100", "200", "300", "400", "500", "600", "700", "800", "900", "1000"];
    for hz in valid_hz.iter() {
        let result = gps.pmtk_220_set_nmea_updaterate(hz);
        println!("{}Hz: {:?}", (1000_f32 / hz.parse::<f32>().unwrap()), result);
        if result == Pmtk001Ack::Success {
            break;
        }
    }

    let stdout = stdout();
    let mut handle = stdout.lock();

    let mut count = 0;
    loop {
        let gps_values = gps.update();
        let sats_found = match gps_values {
            GpsSentence::GGA(sentence) => sentence.satellites_used,
            GpsSentence::NoConnection => {
                println!("GPS not connected");
                0
            }
            _ => 0,
        };

        handle.write_all(format!("\rGPS satellites found: {}", sats_found).as_bytes()).unwrap();
        handle.flush().unwrap();
        thread::sleep(Duration::from_millis(100));
        count += 1;
        if count > 5 {
            if sats_found > 6 {
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

    for i in (1..seconds_to_wait + 1).rev() {
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

    let mut cam = Command::new("raspivid")
        .arg("-o")
        .arg(vid_name)
        .arg("-t")
        .arg("0").stdin(Stdio::piped()).spawn().unwrap();

    println!("Initialise servo");
    feldspar_parachute(0, vec![[500, 500]]);

    println!("Checking Gps");
    gps_checker();

    println!("Starting gps...");
    let (gps_tx, gps_rx) = mpsc::channel();
    let _gps_thread = thread::spawn(move || {
        let max_alt = feldspar_gps(gps_file_name.as_str(), gps_rx);
        println!("Maximum altitude: {}", max_alt);
    });

    println!("Press enter to begin launch countdown.");
    let mut s = String::new();
    let _stdin = io::stdin().read_line(&mut s).unwrap();


    for i in (1..11).rev() {
        println!("{}", i);
        thread::sleep(Duration::from_secs(1));
    }
    println!("Launch!");
    let parachute_thread = thread::spawn(move || {
        feldspar_parachute(deploy_delay, vec![[2500, 1000], [500, 500]]);
        println!("Deployed!");
    });

    for i in 1..recording_duration {
        println!("-{}", i);
        thread::sleep(Duration::from_secs(1));
    }

    parachute_thread.join().unwrap();
    cam.kill().unwrap();
    let _ = gps_tx.send(true);
}
