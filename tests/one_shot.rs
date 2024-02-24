extern crate embedded_hal_mock as hal;
#[macro_use]
extern crate nb;
extern crate opt300x;
use hal::eh1::i2c::Transaction as I2cTrans;
use opt300x::Status;

mod common;
use self::common::{destroy, new_opt3001, BitFlags as BF, Register as Reg, CFG_DEFAULT, DEV_ADDR};

#[test]
fn read_measurement() {
    let value = 0x789A;
    let transactions = [
        I2cTrans::write(
            DEV_ADDR,
            vec![
                Reg::CONFIG,
                ((CFG_DEFAULT | BF::MODE0) >> 8) as u8,
                CFG_DEFAULT as u8,
            ],
        ),
        I2cTrans::write_read(
            DEV_ADDR,
            vec![Reg::CONFIG],
            vec![((CFG_DEFAULT) >> 8) as u8, CFG_DEFAULT as u8],
        ),
        I2cTrans::write_read(
            DEV_ADDR,
            vec![Reg::CONFIG],
            vec![
                ((CFG_DEFAULT | BF::OVF) >> 8) as u8,
                (CFG_DEFAULT | BF::CRF | BF::FH) as u8,
            ],
        ),
        I2cTrans::write_read(
            DEV_ADDR,
            vec![Reg::RESULT],
            vec![(value >> 8) as u8, (value & 0xFF) as u8],
        ),
    ];
    let mut sensor = new_opt3001(&transactions);
    let measurement = block!(sensor.read_lux()).unwrap();

    assert!(measurement.result > 2818.56 - 0.5);
    assert!(measurement.result < 2818.56 + 0.5);
    assert_eq!(
        Status {
            has_overflown: true,
            conversion_ready: true,
            was_too_high: true,
            was_too_low: false,
        },
        measurement.status
    );
    destroy(sensor);
}
