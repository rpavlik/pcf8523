use chrono::prelude::*;
use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

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
    pub fn new(i2cpath: &str) -> Pcf8523 {
        let i2caddr = 0x68;
        let mut dev = LinuxI2CDevice::new(i2cpath, i2caddr).unwrap();
        println!("{}", dev.smbus_read_byte_data(0x04).unwrap());
        Pcf8523{
            dev: dev,
        }
    }

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
