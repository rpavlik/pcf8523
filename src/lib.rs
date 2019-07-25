//! # Pcf8523
//!
//! `Pcf8523` is a crate which abstracts away managing a PCF8523 device on an
//! I2C bus. You can read the time and write the time, and someday in the future
//! do other configuration tasks as well.

use chrono::prelude::*;
use i2cdev::core::*;
use i2cdev::linux::LinuxI2CDevice;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

fn bcd_decode(x: u8) -> u32 {
    ((((x & 0xF0) >> 4) * 10) + (x & 0x0F)) as u32
}

fn bcd_encode(x: u32) -> u8 {
    if x >= 100 {
        panic!("Tried to BCD encode value {} >= 100", x);
    }
    let lower = x % 10;
    let upper = x / 10;
    (lower | (upper << 4)) as u8
}

pub struct Pcf8523 {
    dev: LinuxI2CDevice,
}

impl Pcf8523 {
    /// Returns a new Pcf8523 using the specified path to an i2c-dev device.
    ///
    /// # Arguments
    ///
    /// * `i2cpath` - Path to the I2C device, e.g. /dev/i2c-1
    ///
    /// # Panics
    ///
    /// This function panics if there is an issue opening the device.
    pub fn new(i2cpath: &str) -> Pcf8523 {
        let i2caddr = 0x68;
        let mut dev = LinuxI2CDevice::new(i2cpath, i2caddr).unwrap();
        println!("{}", dev.smbus_read_byte_data(0x04).unwrap());
        Pcf8523{
            dev: dev,
        }
    }

    /// Returns the time in UTC from the device.
    ///
    /// # Panics
    ///
    /// Panics if there is an issue reading the I2C bus, or if the data stored
    /// on the chip is not a valid UTC time.
    pub fn get_time(&mut self) -> chrono::DateTime<Utc> {
        let fields = self.dev.smbus_read_i2c_block_data(0x03, 7).unwrap();
        let sec = bcd_decode(fields[0]);
        let min = bcd_decode(fields[1]);
        let hour = bcd_decode(fields[2]);
        let day = bcd_decode(fields[3]);
        let mon = bcd_decode(fields[5]);
        let yr = bcd_decode(fields[6]);
        Utc.ymd(2000 + yr as i32, mon, day).and_hms(hour, min, sec)
    }

    /// Programs the given time, in UTC, to the device.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The current year is < 2000 or >= 2100, or
    /// - If there is an error writing to the I2C bus.
    pub fn set_time(&mut self, time: chrono::DateTime<Utc>) {
        let sec = bcd_encode(time.second());
        let min = bcd_encode(time.minute());
        let hour = bcd_encode(time.hour());
        let day = bcd_encode(time.day());

        // chrono has Sunday == 7, PCF8523 has Sunday == 0 (like a sane person)
        let days_since_monday = time.weekday().number_from_monday();
        let dow = bcd_encode(if days_since_monday == 7 { 0 } else { days_since_monday });

        let mon = bcd_encode(time.month());
        let yr = bcd_encode(time.year() as u32 - 2000);
        let data = [sec, min, hour, day, dow, mon, yr];
        self.dev.smbus_write_i2c_block_data(0x03, &data).unwrap();
    }
}
