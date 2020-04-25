use std::fs::OpenOptions;
use std::thread;

use adafruit_gps::gps::{GetGpsData, Gps, open_port};
use adafruit_gps::PMTK::send_pmtk::SendPmtk;

fn main() {
    let port = open_port("/dev/serial0");
    let mut gps = Gps { port };

    gps.pmtk_314_api_set_nmea_output(0, 0, 1, 1, 1, 1, 1);
    let _pmtk001 = gps.pmtk_220_set_nmea_updaterate("600");

    loop {
        let gps_values = gps.update();

        println!(format!(
                "{:?},{:?},{:?},{:?},{:?},{:?}\n",
                gps_values.utc,
                gps_values.latitude,
                gps_values.longitude,
                gps_values.speed_kph,
                gps_values.geoidal_spe,
                gps_values.altitude
            )
        )
    }
}