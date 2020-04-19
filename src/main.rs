use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process::Command;
use std::thread;
use std::time::{Duration};

use adafruit_gps::gps::{GetGpsData, Gps, open_port};
use adafruit_gps::PMTK::send_pmtk::SendPmtk;

fn feldspar_gps() {
    let port = open_port("/dev/serial0");
    let mut gps = Gps { port };

    gps.pmtk_314_api_set_nmea_output(0, 0, 1, 1, 1, 1, 1);

    let _file = OpenOptions::new().write(true)
        .create_new(true)
        .open("gps_data.txt");

    let mut gps_file = OpenOptions::new().append(true).open("gps_data.txt")  // fails if no file.
        .expect("cannot open file");

    loop {
        let gps_values = gps.update();

        gps_file.write_all(
            format!("{:?},{:?},{:?},{:?},{:?},{:?}\n",
                    gps_values.utc, gps_values.latitude,
                    gps_values.longitude, gps_values.speed_kph, gps_values.geoidal_spe,
                    gps_values.altitude)
                .as_bytes()).expect("Failed to write line");
    }
}


fn feldspar_cam(seconds: u64) {
    let mili = Duration::from_secs(seconds).as_millis().to_string();
    let _c = Command::new("raspivid").arg("-o").arg("video.h264").arg("-t").arg(mili.as_str()).output().expect("Camera failed to open.");
}


fn main() {
    println!("Standby...");

    println!("Press enter to begin launch countdown.");

    thread::spawn(|| {
        dbg!("Spawn gps");
        feldspar_gps()
    });
    thread::spawn(|| {
        dbg!("Spawn cam");
        feldspar_cam(100);
    });

    let mut s = String::new();
    let _stdin = io::stdin().read_line(&mut s).unwrap();
    for i in (0..10).rev(){
        println!("{}", i);
        thread::sleep(Duration::from_secs(1));
    }
    println!("Launch!")

}

