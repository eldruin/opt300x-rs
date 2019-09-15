extern crate embedded_hal_mock as hal;
extern crate opt300x;
use hal::i2c::Transaction as I2cTrans;

mod common;
use common::{destroy, new_opt3001, Register as Reg, DEV_ADDR};

#[test]
fn can_create_and_destroy() {
    let sensor = new_opt3001(&[]);
    destroy(sensor);
}

#[test]
fn can_read_device_id() {
    let transactions = [I2cTrans::write_read(
        DEV_ADDR,
        vec![Reg::DEVICE_ID],
        vec![0xAB, 0xCD],
    )];
    let mut sensor = new_opt3001(&transactions);
    assert_eq!(0xABCD, sensor.get_device_id().unwrap());
    destroy(sensor);
}

#[test]
fn can_read_manufacturer_id() {
    let transactions = [I2cTrans::write_read(
        DEV_ADDR,
        vec![Reg::MANUFACTURER_ID],
        vec![0xAB, 0xCD],
    )];
    let mut sensor = new_opt3001(&transactions);
    assert_eq!(0xABCD, sensor.get_manufacturer_id().unwrap());
    destroy(sensor);
}

macro_rules! read_lux_test {
    ($name:ident, $register:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let transactions = [I2cTrans::write_read(
                DEV_ADDR,
                vec![Reg::RESULT],
                vec![($register >> 8) as u8, ($register & 0xFF) as u8],
            )];
            let mut sensor = new_opt3001(&transactions);
            let result = sensor.read_lux().unwrap();
            assert!(result > $expected - 0.5);
            assert!(result < $expected + 0.5);
            destroy(sensor);
        }
    };
}

read_lux_test!(lux_0_01, 0x01, 0.01);
read_lux_test!(lux_40, 0xFFF, 40.95);
read_lux_test!(lux_88, 0x3456, 88.80);
read_lux_test!(lux_2818, 0x789A, 2818.56);
read_lux_test!(lux_5242_1, 0x8800, 5242.88);
read_lux_test!(lux_5242_2, 0x9400, 5242.88);
read_lux_test!(lux_5242_3, 0xA200, 5242.88);
read_lux_test!(lux_5242_4, 0xB100, 5242.88);
read_lux_test!(lux_20, 0xB001, 20.48);
read_lux_test!(lux_83k, 0xBFFF, 83865.60);

macro_rules! read_raw_test {
    ($name:ident, $register:expr, ($exp:expr, $mantissa:expr)) => {
        #[test]
        fn $name() {
            let transactions = [I2cTrans::write_read(
                DEV_ADDR,
                vec![Reg::RESULT],
                vec![($register >> 8) as u8, ($register & 0xFF) as u8],
            )];
            let mut sensor = new_opt3001(&transactions);
            let result = sensor.read_raw().unwrap();
            assert_eq!(($exp, $mantissa), result);
            destroy(sensor);
        }
    };
}

read_raw_test!(raw_0_01, 0x01, (0, 0x01));
read_raw_test!(raw_40, 0xFFF, (0, 0xFFF));
read_raw_test!(raw_88, 0x3456, (0x3, 0x456));
read_raw_test!(raw_2818, 0x789A, (0x7, 0x89A));
read_raw_test!(raw_5242_1, 0x8800, (0x8, 0x800));
read_raw_test!(raw_5242_2, 0x9400, (0x9, 0x400));
read_raw_test!(raw_5242_3, 0xA200, (0xA, 0x200));
read_raw_test!(raw_5242_4, 0xB100, (0xB, 0x100));
read_raw_test!(raw_20, 0xB001, (0xB, 0x01));
read_raw_test!(raw_83k, 0xBFFF, (0xB, 0xFFF));
