use hal::i2c::{Mock as I2cMock, Transaction as I2cTrans};
use opt300x::{ic, mode, Opt300x, SlaveAddr};

pub const DEV_ADDR: u8 = 0b100_0100;
pub const CFG_DEFAULT: u16 = 0xC810;

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

pub struct BitFlags;
#[allow(unused)]
impl BitFlags {
    pub const CT: u16 = 1 << 11;
    pub const MODE1: u16 = 1 << 10;
    pub const MODE0: u16 = 1 << 9;
    pub const OVF: u16 = 1 << 8;
    pub const POL: u16 = 1 << 3;
    pub const ME: u16 = 1 << 2;
}

#[allow(unused)]
pub fn new_opt3001(transactions: &[I2cTrans]) -> Opt300x<I2cMock, ic::Opt3001, mode::OneShot> {
    Opt300x::new_opt3001(I2cMock::new(&transactions), SlaveAddr::default())
}

pub fn destroy<IC, MODE>(sensor: Opt300x<I2cMock, IC, MODE>) {
    sensor.destroy().done();
}

#[macro_export]
macro_rules! assert_invalid_input_data {
    ($result:expr) => {
        match $result {
            Err(Error::InvalidInputData) => (),
            _ => panic!("InvalidInputData error not returned."),
        }
    };
}

#[macro_export]
macro_rules! set_invalid_test {
    ($name:ident, $create_method:ident, $destroy_method:ident, $method:ident $(, $value:expr)*) => {
        #[test]
        fn $name() {
            let mut dev = $create_method(&[]);
            assert_invalid_input_data!(dev.$method($($value),*));
            $destroy_method(dev);
        }
    };
}
