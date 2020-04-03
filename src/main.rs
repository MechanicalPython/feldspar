use std::thread;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime};

mod feldspar_camera {
    use std::fs::File;
    use std::io::Write;

    use rascam::SimpleCamera;

    pub fn init_camera() -> SimpleCamera {
        // May need to be a bit of a werid workaround by taking 25 photos per second and stitching them
        // together afterwards.
        let info = rascam::info().unwrap();
        let mut camera = SimpleCamera::new(info.cameras[0].clone()).unwrap();
        camera.activate().unwrap();
        return camera;
    }

    pub fn take_photo(mut camera: SimpleCamera, image_path: &str) {
        let photo = camera.take_one().unwrap();
        File::create(image_path).unwrap().write_all(&photo).unwrap()
    }
}

mod feldspar_gps {
    use adafruit_gps::{Gps, GpsArgValues, open_port};

    pub fn init_gps() -> Gps {
        let mut gps = Gps { port: open_port("/dev/serial0") };

        gps.send_command("PMTK314,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0");
        gps.send_command("PMTK220,500");  // 2Hz info rate now.
        return feldspar_gps;
    }

    pub fn get_new_gps_vals(mut gps: Gps) -> GpsArgValues {
        return gps.update(GpsArgValues::default());
    }
}

fn feldspar_gps() {
    let mut gps = feldspar_gps::init_gps();
    let file = OpenOptions::new().write(true)
        .create_new(true)
        .open("gps_data.txt");

    let mut gps_file = OpenOptions::new().append(true).open("gps_data.txt")  // fails if no file.
        .expect("cannot open file");
    let mut last_gps_reading = SystemTime::now();
    loop {
        // GPS
        if last_gps_reading.elapsed().unwrap().as_millis() >= 500 {
            last_gps_reading = SystemTime::now();
            let gps_vals = feldspar_gps::get_new_gps_vals(gps);
            gps_file.write_all(format!("{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?}\n",
                                       gps_vals.timestamp, gps_vals.latitude,
                                       gps_vals.longitude, gps_vals.fix_quality,
                                       gps_vals.satellites, gps_vals.altitude_m,
                                       gps_vals.speed_knots, gps_vals.horizontal_dilution)
                .as_bytes())
        }
    }
}

fn feldspar_cam() {
    let mut camera = feldspar_camera::init_camera();
    let mut last_photo = SystemTime::now();
    let mut photo_num = 0;
    loop {
        if last_photo.elapsed().unwrap().as_millis() >=40 {  // 25 fps
            feldspar_camera::take_photo(*camera, format!("feldspar_cam/{}.jpeg", photo_num).as_ref());
        }
    }
}

fn main() {
    thread::spawn(|| {
        feldspar_gps()
    });
    thread::spawn(|| {
        feldspar_cam()
    });
}
