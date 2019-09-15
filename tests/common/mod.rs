use hal::i2c::{Mock as I2cMock, Transaction as I2cTrans};
use opt300x::{ic, Opt300x, SlaveAddr};

pub const DEV_ADDR: u8 = 0b100_0100;

pub struct Register;
#[allow(unused)]
impl Register {
    pub const RESULT: u8 = 0x00;
    pub const CONFIG: u8 = 0x01;
    pub const LOW_LIMIT: u8 = 0x02;
    pub const HIGH_LIMIT: u8 = 0x03;
    pub const MANUFACTURER_ID: u8 = 0x7E;
    pub const DEVICE_ID: u8 = 0x7F;
}

#[allow(unused)]
pub fn new_opt3001(transactions: &[I2cTrans]) -> Opt300x<I2cMock, ic::Opt3001> {
    Opt300x::new_opt3001(I2cMock::new(&transactions), SlaveAddr::default())
}

pub fn destroy<IC>(sensor: Opt300x<I2cMock, IC>) {
    sensor.destroy().done();
}
