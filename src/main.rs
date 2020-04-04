use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::{Duration, SystemTime};

use adafruit_gps::{Gps, GpsArgValues, open_port};
use rascam::SimpleCamera;

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

            if gps_values.fix_quality != Some(1) {
                gps_file.write_all(format!("{:?} -- No fix\n", gps_values.timestamp).as_bytes()
                ).expect("Failed to write file");
            } else {
                gps_file.write_all(format!("{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?}\n",
                                           gps_values.timestamp, gps_values.latitude,
                                           gps_values.longitude, gps_values.fix_quality,
                                           gps_values.satellites, gps_values.altitude_m,
                                           gps_values.speed_knots, gps_values.horizontal_dilution)
                    .as_bytes()).expect("Failed to write");
            }
        }
    }
}

fn feldspar_cam() {
    let info = rascam::info().unwrap();
    let mut camera = SimpleCamera::new(info.cameras[0].clone()).unwrap();
    camera.activate().unwrap();

    let sleep_duration = Duration::from_millis(2000);
    thread::sleep(sleep_duration);

    let mut last_photo = SystemTime::now();
    let mut photo_num = 0;
    loop {
        if last_photo.elapsed().unwrap().as_millis() >= 40 {  // 25 fps
            last_photo = SystemTime::now();
            photo_num += 1;

            let photo = camera.take_one().unwrap();
            let photo_path = format!("feldspar_cam/{}.jpeg", photo_num);
            let photo_path = photo_path.as_str();
            File::create(photo_path).unwrap().write_all(&photo).unwrap();
        }
    }
}


fn main() {
    thread::spawn(|| {
        dbg!("Spawn gps");
        feldspar_gps()
    });
    dbg!("Spawn cam");
    feldspar_cam();
}
