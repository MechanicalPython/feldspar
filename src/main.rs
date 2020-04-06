use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use std::thread;
use std::time::SystemTime;

use adafruit_gps::{Gps, GpsArgValues, open_port};

fn feldspar_gps() {
    let mut gps = Gps { port: open_port("/dev/serial0") };
    let mut gps_values = GpsArgValues::default();

    gps.send_command("PMTK314,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0");
    gps.send_command("PMTK220,500");

    let _file = OpenOptions::new().write(true)
        .create_new(true)
        .open("gps_data.txt");

    let mut gps_file = OpenOptions::new().append(true).open("gps_data.txt")  // fails if no file.
        .expect("cannot open file");

    let mut last_gps_reading = SystemTime::now();

    loop {
        if last_gps_reading.elapsed().unwrap().as_millis() >= 500 {
            gps_values = gps.update(gps_values);
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

fn feldspar_cam() {
    let c = Command::new("raspivid").arg("-o").arg("video.h264").arg("-t").arg("300000").output().expect("Camera failed");
    // 1000 = 1 second. 300000 is 5 mins.
    dbg!(c);
}


fn main() {
    thread::spawn(|| {
        dbg!("Spawn gps");
        feldspar_gps()
    });
    dbg!("Spawn cam");
    feldspar_cam();  // When this command is done, the main() closes so the gps stops.
}
