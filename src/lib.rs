//! # Pcf8523
//!
//! `Pcf8523` is a crate which abstracts away managing a PCF8523 device on an
//! I2C bus. You can read the time and write the time, and someday in the future
//! do other configuration tasks as well.
#![no_std]

#![cfg_attr(not(feature = "std"), no_std)]
extern crate embedded_hal as hal;

use chrono::{Datelike, TimeZone, Timelike, Utc};
use hal::i2c::blocking::{Write, WriteRead};

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

const ADDRESS: u8 = 0x68;

pub struct Pcf8523<I2C> {
    i2c: I2C,
}

impl<I2C> Pcf8523<I2C> {
    /// Returns a new Pcf8523 using the specified i2c device.
    ///
    /// # Arguments
    ///
    /// * `i2c` - A struct implementing blocking I2C traits
    pub fn new(i2c: I2C) -> Self {
        Pcf8523 { i2c: i2c }
    }
}

impl<I2C: WriteRead> Pcf8523<I2C> {
    /// Returns the time in UTC from the device.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an issue reading the I2C bus, or if the data stored
    /// on the chip is not a valid UTC time.
    pub fn get_time(&mut self) -> Result<chrono::DateTime<Utc>, I2C::Error> {
        let out_buf = [0x03u8];
        let mut fields = [0u8; 7];

        self.i2c.write_read(ADDRESS, &out_buf, &mut fields)?;
        let sec = bcd_decode(fields[0]);
        let min = bcd_decode(fields[1]);
        let hour = bcd_decode(fields[2]);
        let day = bcd_decode(fields[3]);
        let mon = bcd_decode(fields[5]);
        let yr = bcd_decode(fields[6]);
        Ok(Utc.ymd(2000 + yr as i32, mon, day).and_hms(hour, min, sec))
    }
}

impl<I2C: Write> Pcf8523<I2C> {
    /// Programs the given time, in UTC, to the device.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The current year is < 2000 or >= 2100, or
    /// - If there is an error writing to the I2C bus.
    pub fn set_time(&mut self, time: chrono::DateTime<Utc>) -> Result<(), I2C::Error> {
        let sec = bcd_encode(time.second());
        let min = bcd_encode(time.minute());
        let hour = bcd_encode(time.hour());
        let day = bcd_encode(time.day());

        // chrono has Sunday == 7, PCF8523 has Sunday == 0 (like a sane person)
        let days_since_monday = time.weekday().number_from_monday();
        let dow = bcd_encode(if days_since_monday == 7 {
            0
        } else {
            days_since_monday
        });

        let mon = bcd_encode(time.month());
        let yr = bcd_encode(time.year() as u32 - 2000);
        let data = [sec, min, hour, day, dow, mon, yr];
        self.i2c.write(ADDRESS, &data)
    }
}
