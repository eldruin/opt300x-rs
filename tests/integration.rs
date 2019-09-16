extern crate embedded_hal_mock as hal;
extern crate opt300x;
use hal::i2c::Transaction as I2cTrans;
use opt300x::{Error, FaultCount, InterruptPinPolarity, LuxRange};

mod common;
use self::common::{destroy, new_opt3001, BitFlags as BF, Register as Reg, CFG_DEFAULT, DEV_ADDR};

#[test]
fn can_create_and_destroy() {
    let sensor = new_opt3001(&[]);
    destroy(sensor);
}

macro_rules! get_test {
    ($name:ident, $method:ident, $register:ident, $value:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let transactions = [I2cTrans::write_read(
                DEV_ADDR,
                vec![Reg::$register],
                vec![($value >> 8) as u8, ($value & 0xFF) as u8],
            )];
            let mut sensor = new_opt3001(&transactions);
            let result = sensor.$method().unwrap();
            assert_eq!($expected, result);
            destroy(sensor);
        }
    };
}

get_test!(can_read_dev_id, get_device_id, DEVICE_ID, 0xABCD, 0xABCD);
get_test!(
    can_read_manuf_id,
    get_manufacturer_id,
    MANUFACTURER_ID,
    0xABCD,
    0xABCD
);

macro_rules! read_lux_test {
    ($name:ident, $value:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let transactions = [
                I2cTrans::write(
                    DEV_ADDR,
                    vec![
                        Reg::CONFIG,
                        ((CFG_DEFAULT | BF::MODE0 | BF::MODE1) >> 8) as u8,
                        CFG_DEFAULT as u8,
                    ],
                ),
                I2cTrans::write_read(
                    DEV_ADDR,
                    vec![Reg::RESULT],
                    vec![($value >> 8) as u8, ($value & 0xFF) as u8],
                ),
            ];
            let sensor = new_opt3001(&transactions);
            let mut sensor = sensor.into_continuous().ok().unwrap();
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
    ($name:ident, $value:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let transactions = [
                I2cTrans::write(
                    DEV_ADDR,
                    vec![
                        Reg::CONFIG,
                        ((CFG_DEFAULT | BF::MODE0 | BF::MODE1) >> 8) as u8,
                        CFG_DEFAULT as u8,
                    ],
                ),
                I2cTrans::write_read(
                    DEV_ADDR,
                    vec![Reg::RESULT],
                    vec![($value >> 8) as u8, ($value & 0xFF) as u8],
                ),
            ];
            let sensor = new_opt3001(&transactions);
            let mut sensor = sensor.into_continuous().ok().unwrap();
            let result = sensor.read_raw().unwrap();
            assert_eq!($expected, result);
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

get_test!(overflow, has_overflown, CONFIG, BF::OVF, true);
get_test!(no_overflow, has_overflown, CONFIG, 0, false);

macro_rules! cfg_test {
    ($name:ident, $method:ident, $value:expr $(, $arg:expr)*) => {
        #[test]
        fn $name() {
            let transactions = [I2cTrans::write(
                DEV_ADDR,
                vec![Reg::CONFIG, ($value >> 8) as u8, $value as u8],
            )];
            let mut sensor = new_opt3001(&transactions);
            sensor.$method($($arg),*).unwrap();
            destroy(sensor);
        }
    };
}

cfg_test!(faultc1, set_fault_count, CFG_DEFAULT, FaultCount::One);
cfg_test!(faultc2, set_fault_count, CFG_DEFAULT | 1, FaultCount::Two);
cfg_test!(faultc4, set_fault_count, CFG_DEFAULT | 2, FaultCount::Four);
cfg_test!(faultc8, set_fault_count, CFG_DEFAULT | 3, FaultCount::Eight);

cfg_test!(
    int_pin_polarity_low,
    set_interrupt_pin_polarity,
    CFG_DEFAULT,
    InterruptPinPolarity::Low
);
cfg_test!(
    int_pin_polarity_high,
    set_interrupt_pin_polarity,
    CFG_DEFAULT | BF::POL,
    InterruptPinPolarity::High
);

#[test]
fn can_change_mode() {
    let transactions = [
        I2cTrans::write(
            DEV_ADDR,
            vec![
                Reg::CONFIG,
                ((CFG_DEFAULT | BF::MODE0 | BF::MODE1) >> 8) as u8,
                CFG_DEFAULT as u8,
            ],
        ),
        I2cTrans::write(
            DEV_ADDR,
            vec![Reg::CONFIG, (CFG_DEFAULT >> 8) as u8, CFG_DEFAULT as u8],
        ),
        I2cTrans::write(
            DEV_ADDR,
            vec![
                Reg::CONFIG,
                ((CFG_DEFAULT | BF::MODE0 | BF::MODE1) >> 8) as u8,
                CFG_DEFAULT as u8,
            ],
        ),
    ];
    let sensor = new_opt3001(&transactions);
    let sensor = sensor.into_continuous().ok().unwrap();
    let sensor = sensor.into_one_shot().ok().unwrap();
    let sensor = sensor.into_continuous().ok().unwrap();
    destroy(sensor);
}

set_invalid_test!(
    too_high_lux_range,
    new_opt3001,
    destroy,
    set_lux_range,
    LuxRange::Manual(0b1100)
);
cfg_test!(
    set_lux_range_auto,
    set_lux_range,
    CFG_DEFAULT,
    LuxRange::Auto
);
cfg_test!(
    set_lux_range_manual_0,
    set_lux_range,
    CFG_DEFAULT & 0x0FFF,
    LuxRange::Manual(0)
);
cfg_test!(
    set_lux_range_manual_max,
    set_lux_range,
    CFG_DEFAULT & 0x0FFF | 0b1011 << 12,
    LuxRange::Manual(0b1011)
);
