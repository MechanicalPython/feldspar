use adafruit_gps::gps::{Gps, GpsSentence, open_port};

fn main() {
    let port = open_port("/dev/serial0", 57600);
    let mut gps = Gps { port };

    gps.pmtk_314_api_set_nmea_output(0, 0, 0, 1, 0, 0, 1);
    let _pmtk001 = gps.pmtk_220_set_nmea_updaterate("100");

    loop {
        let gps_values = gps.update();
        match gps_values {
            GpsSentence::GGA(sentence) => {
                println!(
                    "{:?},{:?},{:?},{:?},{:?},{:?}\n",
                    sentence.utc,
                    sentence.lat,
                    sentence.long,
                    sentence.geoidal_sep,
                    sentence.msl_alt,
                );
            }
            _ => {}
        }
    }
}