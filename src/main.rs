use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command};
use std::thread;
use std::time::{Duration, SystemTime};

use adafruit_gps::gps::{GetGpsData, Gps, open_port};
use adafruit_gps::PMTK::send_pmtk::SendPmtk;

fn feldspar_gps(capture_duration: u64, file_name: &str) {

    let port = open_port("/dev/serial0");
    let mut gps = Gps { port };

    gps.pmtk_314_api_set_nmea_output(0, 0, 1, 1, 1, 1, 1);

    let _file = OpenOptions::new().write(true)
        .create_new(true)
        .open(file_name);

    let mut gps_file = OpenOptions::new().append(true).open(file_name)  // fails if no file.
        .expect("cannot open file");
    let start_time = SystemTime::now();
    while start_time.elapsed().unwrap() < Duration::from_secs(capture_duration) {
        let gps_values = gps.update();

        gps_file.write_all(
            format!("{:?},{:?},{:?},{:?},{:?},{:?}\n",
                    gps_values.utc, gps_values.latitude,
                    gps_values.longitude, gps_values.speed_kph, gps_values.geoidal_spe,
                    gps_values.altitude)
                .as_bytes()).expect("Failed to write line");
    }
}


fn feldspar_cam(seconds: u64, vid_file: &str) {
    let mili = Duration::from_secs(seconds).as_millis().to_string();
    let _c = Command::new("raspivid").arg("-o").arg(vid_file).arg("-t").arg(mili.as_str()).output().expect("Camera failed to open.");
}


fn main() {
    let args: Vec<String> = env::args().collect();

    let launch_duration: &str = args.get(1).expect("Please enter an instrument recording time (seconds)");
    let launch_duration = launch_duration.parse::<u64>().expect("Please enter a valid integer for launch duration");
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

    let gps_thread = thread::spawn(move|| {
        println!("Starting gps...");
        feldspar_gps(launch_duration, gps_file_name.as_str())
    });

    let cam_thread = thread::spawn(move|| {
        println!("Starting camera...");
        feldspar_cam(launch_duration, vid_name.as_str());
    });

    for i in (1..11).rev() {
        println!("{}", i);
        thread::sleep(Duration::from_secs(1));
    }
    println!("Launch!");
    for i in (0..launch_duration - 10).rev() {
        println!("{}", i);
    }

    cam_thread.join().unwrap();
    gps_thread.join().unwrap();
}

