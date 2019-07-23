use pcf8523::{Pcf8523};
use chrono::prelude::*;

fn main() {
    let mut dev = Pcf8523::new("/dev/i2c-1");
    println!("PCF8523 time: {:?}", dev.get_time());

    // Actual time
    let now = Utc::now();
    println!("System time: {:?}", now);

    dev.set_time(now);
    println!("PCF8523 time after setting: {:?}", dev.get_time());
}
