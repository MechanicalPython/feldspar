use adafruit_gps::gps::{Gps, GpsSentence, open_port};

fn main() {
    let port = open_port("/dev/serial0", 9600);
    let mut gps = Gps { port };

    gps.pmtk_314_api_set_nmea_output(0, 0, 0, 1, 0, 0, 1);

    loop {
        let gps_values = gps.update();
        match gps_values {
            GpsSentence::GGA(sentence) => {
                println!(
                    "{:?},{:?},{:?},{:?},{:?}\n",
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