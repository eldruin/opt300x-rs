extern crate linux_embedded_hal as hal;
extern crate opt300x;
use opt300x::{Opt300x, SlaveAddr};

fn main() {
    let dev = hal::I2cdev::new("/dev/i2c-1").unwrap();
    let address = SlaveAddr::default();
    let sensor = Opt300x::new_opt3001(dev, address);
    let mut sensor = sensor.into_continuous().ok().unwrap();
    loop {
        let lux = sensor.read_lux().unwrap();
        println!("lux: {:2}", lux);
    }
}
