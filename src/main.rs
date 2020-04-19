use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use std::thread;
use std::time::{SystemTime, Duration};

use adafruit_gps::gps::{GetGpsData, Gps, open_port};
use adafruit_gps::PMTK::send_pmtk::SendPmtk;

fn feldspar_gps() {
    let port = open_port("/dev/serial0");
    let mut gps = Gps {port};

    gps.pmtk_314_api_set_nmea_output(0,0,1,1,1,1,1);

    let _file = OpenOptions::new().write(true)
        .create_new(true)
        .open("gps_data.txt");

    let mut gps_file = OpenOptions::new().append(true).open("gps_data.txt")  // fails if no file.
        .expect("cannot open file");

    let mut last_gps_reading = SystemTime::now();

    loop {
        if last_gps_reading.elapsed().unwrap().as_millis() >= 500 {
            gps_values = gps.update();
            last_gps_reading = SystemTime::now();
            // println!("{:?}", gps_values);
            if gps_values.fix_quality != Some(1) {
                gps_file.write_all(format!("{:?} -- No fix\n", gps_values.timestamp).as_bytes()
                ).expect("Failed to write file");
            } else {
                gps_file.write_all(
                    format!("{:?},{:?},{:?},{:?},{:?},{:?},{:?}\n",
                            gps_values.timestamp, gps_values.latitude,
                             gps_values.longitude, gps_values.fix_quality,
                             gps_values.satellites, gps_values.altitude_m,
                             gps_values.horizontal_dilution)
                        .as_bytes()).expect("Failed to write line");
            }
        }
    }
}


fn feldspar_cam(seconds: u64) {
    let mili = Duration::from_secs(seconds).as_millis().to_string();
    let c = Command::new("raspivid").arg("-o").arg("video.h264").arg("-t").arg(mili.as_str()).output().expect("Camera failed to open.");
}


fn main() {
    thread::spawn(|| {
        dbg!("Spawn gps");
        feldspar_gps()
    });
    dbg!("Spawn cam");
    feldspar_cam(60);  // When this command is done, the main() closes so the gps stops.
}

