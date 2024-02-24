extern crate embedded_hal_mock as hal;
extern crate opt300x;
use hal::eh1::i2c::{Mock as I2cMock, Transaction as I2cTrans};
use opt300x::{
    ComparisonMode, Error, FaultCount, IntegrationTime, InterruptPinPolarity, LuxRange, Opt300x,
    SlaveAddr, Status,
};

mod common;
use self::common::{destroy, new_opt3001, BitFlags as BF, Register as Reg, CFG_DEFAULT, DEV_ADDR};

macro_rules! create_destroy_test {
    ($name:ident, $method:ident) => {
        #[test]
        fn $name() {
            let sensor = Opt300x::$method(I2cMock::new(&[]), SlaveAddr::default());
            destroy(sensor);
        }
    };
}
create_destroy_test!(create_and_destroy_opt3001, new_opt3001);
create_destroy_test!(create_and_destroy_opt3002, new_opt3002);
create_destroy_test!(create_and_destroy_opt3004, new_opt3004);
create_destroy_test!(create_and_destroy_opt3006, new_opt3006);

#[test]
fn create_and_destroy_opt3007() {
    let sensor = Opt300x::new_opt3007(I2cMock::new(&[]));
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
read_lux_test!(lux_83k, 0xBFFF, 83_865.6);

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

get_test!(
    status_overflow,
    read_status,
    CONFIG,
    BF::OVF,
    Status {
        has_overflown: true,
        conversion_ready: false,
        was_too_high: false,
        was_too_low: false,
    }
);

get_test!(
    status_all_false,
    read_status,
    CONFIG,
    0,
    Status {
        has_overflown: false,
        conversion_ready: false,
        was_too_high: false,
        was_too_low: false,
    }
);

get_test!(
    status_conversion_ready,
    read_status,
    CONFIG,
    BF::CRF,
    Status {
        has_overflown: false,
        conversion_ready: true,
        was_too_high: false,
        was_too_low: false,
    }
);

get_test!(
    status_too_high,
    read_status,
    CONFIG,
    BF::FH,
    Status {
        has_overflown: false,
        conversion_ready: false,
        was_too_high: true,
        was_too_low: false,
    }
);

get_test!(
    status_too_low,
    read_status,
    CONFIG,
    BF::FL,
    Status {
        has_overflown: false,
        conversion_ready: false,
        was_too_high: false,
        was_too_low: true,
    }
);

get_test!(
    status_all_true,
    read_status,
    CONFIG,
    BF::OVF | BF::CRF | BF::FH | BF::FL,
    Status {
        has_overflown: true,
        conversion_ready: true,
        was_too_high: true,
        was_too_low: true,
    }
);

macro_rules! set_test {
    ($name:ident, $method:ident, $register:ident, $value:expr $(, $arg:expr)*) => {
        #[test]
        fn $name() {
            let transactions = [I2cTrans::write(
                DEV_ADDR,
                vec![Reg::$register, ($value >> 8) as u8, $value as u8],
            )];
            let mut sensor = new_opt3001(&transactions);
            sensor.$method($($arg),*).unwrap();
            destroy(sensor);
        }
    };
}

macro_rules! cfg_test {
    ($name:ident, $method:ident, $value:expr $(, $arg:expr)*) => {
        set_test!($name, $method, CONFIG, $value $(, $arg)*);
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

cfg_test!(
    comp_mode_latched_window,
    set_comparison_mode,
    CFG_DEFAULT,
    ComparisonMode::LatchedWindow
);

cfg_test!(
    comp_mode_transparent,
    set_comparison_mode,
    CFG_DEFAULT & !BF::L,
    ComparisonMode::TransparentHysteresis
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

cfg_test!(
    set_integration_time_100,
    set_integration_time,
    CFG_DEFAULT & !BF::CT,
    IntegrationTime::Ms100
);

cfg_test!(
    set_integration_time_800,
    set_integration_time,
    CFG_DEFAULT | BF::CT,
    IntegrationTime::Ms800
);

cfg_test!(
    enable_exponent_masking,
    enable_exponent_masking,
    CFG_DEFAULT | BF::ME
);
cfg_test!(
    disable_exponent_masking,
    disable_exponent_masking,
    CFG_DEFAULT & !BF::ME
);
macro_rules! invalid_test {
    ($name:ident, $method:ident $(, $arg:expr)*) => {
        #[test]
        fn $name() {
            let mut sensor = new_opt3001(&[]);
            if let Err(Error::InvalidInputData) = sensor.$method($($arg),*) { }
            else {
                panic!("Should have returned error");
            }
            destroy(sensor);
        }
    };
}

invalid_test!(low_limit_exp_too_big, set_low_limit_raw, 0b1100, 0);
invalid_test!(low_limit_mant_too_big, set_low_limit_raw, 0, 0x1000);
set_test!(
    set_low_limit,
    set_low_limit_raw,
    LOW_LIMIT,
    0xBFFF_u16,
    0xB,
    0xFFF
);

invalid_test!(high_limit_exp_too_big, set_high_limit_raw, 0b1100, 0);
invalid_test!(high_limit_mant_too_big, set_high_limit_raw, 0, 0x1000);
set_test!(
    set_high_limit,
    set_high_limit_raw,
    HIGH_LIMIT,
    0xBFFF_u16,
    0xB,
    0xFFF
);

set_test!(
    enable_end_of_conv,
    enable_end_of_conversion_mode,
    LOW_LIMIT,
    0b11 << 14
);

set_test!(
    disable_end_of_conv,
    disable_end_of_conversion_mode,
    LOW_LIMIT,
    0
);

#[test]
fn configured_low_limit_is_restored_after_disabling_end_of_conv() {
    let low_limit = 0b1010_1010_1010_1010;
    let transactions = [
        I2cTrans::write(
            DEV_ADDR,
            vec![Reg::LOW_LIMIT, (low_limit >> 8) as u8, low_limit as u8],
        ),
        I2cTrans::write(
            DEV_ADDR,
            vec![
                Reg::LOW_LIMIT,
                (low_limit >> 8) as u8 | 0b11 << 6,
                low_limit as u8,
            ],
        ),
        I2cTrans::write(
            DEV_ADDR,
            vec![Reg::LOW_LIMIT, (low_limit >> 8) as u8, low_limit as u8],
        ),
    ];
    let mut sensor = new_opt3001(&transactions);
    sensor
        .set_low_limit_raw((low_limit >> 12) as u8, low_limit & 0xFFF)
        .unwrap();
    sensor.enable_end_of_conversion_mode().unwrap();
    sensor.disable_end_of_conversion_mode().unwrap();
    destroy(sensor);
}
