//! # Read mpu9250
//!
//! ## Connections
//!
//! IMPORTANT: Do *not* use PIN24 / BCM8 / CE0 as the NCS pin
//!
//! SPI interface
//! - PIN1 = 3V3 = VCC
//! - PIN19 = BCM10 = MOSI (SDA)
//! - PIN21 = BCM9 = MISO (AD0)
//! - PIN23 = BCM11 = SCLK (SCL)
//! - PIN22 = BCM25 = NCS
//! - PIN6 = GND = GND
//!
//!
//! Aim: to be able to open the port, read from it when needed, and then close the port.
//! Maybe add code that can close the port if there is an error.

extern crate linux_embedded_hal as hal;
extern crate mpu9250;

use std::thread::sleep;
use std::time::Duration;

use hal::{Delay, Pin, Spidev};
use hal::spidev::{self, SpidevOptions};
use hal::sysfs_gpio::Direction;
use mpu9250::Mpu9250;
use self::mpu9250::{Marg, SpiDevice};


pub struct Mpu {
    pub port: Mpu9250<SpiDevice<Spidev, Pin>, Marg>
}

pub fn open_mpu_port() -> Mpu {
    let mut spi = Spidev::open("/dev/spidev0.0").unwrap();
    let options = SpidevOptions::new().max_speed_hz(1_000_000)
                                      .mode(spidev::SPI_MODE_3)
                                      .build();
    spi.configure(&options).unwrap();

    let ncs = Pin::new(25);
    ncs.export().unwrap();
    sleep(Duration::from_millis(100));  // Seems to fix set_direction permission issue
    while !ncs.is_exported() {}

    ncs.set_direction(Direction::Out).unwrap();  // Permission error here on first run.
    ncs.set_value(1).unwrap();

    let mpu = Mpu9250::marg_default(spi, ncs, &mut Delay).unwrap();
    return Mpu{
        port: mpu,
    };
}

pub fn close_mpu_port() {
    let _ = Pin::new(25).unexport().unwrap();
    sleep(Duration::from_millis(100));
}