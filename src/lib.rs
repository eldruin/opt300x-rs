//! This is a platform agnostic Rust driver for the OPT300x ambient light
//! sensors using the [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//! This driver allows you to:
//! - Read the measurement in lux or raw. See: [`read_lux()`].
//! - Change into continuous measurement mode. See: [`into_continuous()`].
//! - Read the conversion status. See: [`read_status()`].
//! - Set the fault count. See: [`set_fault_count()`].
//! - Set the interrupt pin polarity. See: [`set_interrupt_pin_polarity()`].
//! - Set the comparison mode. See: [`set_comparison_mode()`].
//! - Set the low and high limits. See: [`set_low_limit_raw()`].
//! - Enable and disable end-of-conversion mode. See: [`enable_end_of_conversion_mode()`].
//! - Get the manufacturer ID. See: [`get_manufacturer_id()`].
//! - Get the device ID. See: [`get_device_id()`].
//!
//! [`read_lux()`]: struct.Opt300x.html#method.read_lux
//! [`into_continuous()`]: struct.Opt300x.html#method.into_continuous
//! [`read_status()`]: struct.Opt300x.html#method.read_status
//! [`set_fault_count()`]: struct.Opt300x.html#method.set_fault_count
//! [`set_interrupt_pin_polarity()`]: struct.Opt300x.html#method.set_interrupt_pin_polarity
//! [`set_comparison_mode()`]: struct.Opt300x.html#method.set_comparison_mode
//! [`set_low_limit_raw()`]: struct.Opt300x.html#method.set_low_limit_raw
//! [`enable_end_of_conversion_mode()`]: struct.Opt300x.html#method.enable_end_of_conversion_mode
//! [`get_manufacturer_id()`]: struct.Opt300x.html#method.get_manufacturer_id
//! [`get_device_id()`]: struct.Opt300x.html#method.get_device_id
//!
//! ## The devices
//!
//! This driver is compatible with the devices OPT3001, OPT3002, OPT3004,
//! OPT3006 and OPT3007.
//!
//! The OPT3001 is a sensor that measures the intensity of visible light.
//! The spectral response of the sensor tightly matches the photopic
//! response of the human eye and includes significant infrared rejection.
//!
//! The OPT3001 is a single-chip lux meter, measuring the intensity of
//! light as visible by the human eye. The precision spectral response and
//! strong IR rejection of the device enables the OPT3001 to accurately
//! meter the intensity of light as seen by the human eye regardless of
//! light source. The strong IR rejection also aids in maintaining high
//! accuracy when industrial design calls for mounting the sensor under
//! dark glass for aesthetics. The OPT3001 is designed for systems that
//! create light-based experiences for humans, and an ideal preferred
//! replacement for photodiodes, photoresistors, or other ambient light
//! sensors with less human eye matching and IR rejection.
//!
//! Measurements can be made from 0.01 lux up to 83k lux without manually
//! selecting full-scale ranges by using the built-in, full-scale setting
//! feature. This capability allows light measurement over a 23-bit
//! effective dynamic range.
//!
//! The digital operation is flexible for system integration. Measurements
//! can be either continuous or single-shot. The control and interrupt
//! system features autonomous operation, allowing the processor to sleep
//! while the sensor searches for appropriate wake-up events to report via
//! the interrupt pin. The digital output is reported over an I2C- and
//! SMBus-compatible, two-wire serial interface.
//!
//! The low power consumption and low power-supply voltage capability of the
//! OPT3001 enhance the battery life of battery-powered systems.
//!
//! Datasheets:
//! - [OPT3001](https://www.ti.com/lit/ds/symlink/opt3001.pdf)
//! - [OPT3002](https://www.ti.com/lit/ds/symlink/opt3002.pdf)
//! - [OPT3004](https://www.ti.com/lit/ds/symlink/opt3004.pdf)
//! - [OPT3006](https://www.ti.com/lit/ds/symlink/opt3006.pdf)
//! - [OPT3007](https://www.ti.com/lit/ds/symlink/opt3007.pdf)
//!
//! Application Guide:
//! - [OPT3001 ALS Application Guide](https://www.ti.com/lit/an/sbea002a/sbea002a.pdf)
//!
//! ## Usage examples (see also examples folder)
//!
//! To use this driver, import this crate and an `embedded_hal` implementation,
//! then instantiate the appropriate device.
//!
//! In the following examples an instance of the device OPT3001 will be created.
//! Other devices can be created with similar methods like:
//! `Opt300x::new_opt3002(...)`.
//!
//! ### Create a driver instance for the OPT3001
//!
//! ```no_run
//! extern crate linux_embedded_hal as hal;
//! extern crate opt300x;
//! use opt300x::{Opt300x, SlaveAddr};
//!
//! # fn main() {
//! let dev = hal::I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::default();
//! let sensor = Opt300x::new_opt3001(dev, address);
//! # }
//! ```
//!
//! ### Provide an alternative address
//!
//! ```no_run
//! extern crate linux_embedded_hal as hal;
//! extern crate opt300x;
//! use opt300x::{Opt300x, SlaveAddr};
//!
//! # fn main() {
//! let dev = hal::I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::Alternative(true, false);
//! let mut sensor = Opt300x::new_opt3001(dev, address);
//! # }
//! ```
//!
//! ### Read lux in one-shot measurements
//!
//! ```no_run
//! extern crate linux_embedded_hal as hal;
//! #[macro_use]
//! extern crate nb;
//! extern crate opt300x;
//! use opt300x::{Opt300x, SlaveAddr};
//!
//! # fn main() {
//! let dev = hal::I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::default();
//! let mut sensor = Opt300x::new_opt3001(dev, address);
//! loop {
//!     let m = block!(sensor.read_lux()).unwrap();
//!     println!("lux: {:2}", m.result);
//! }
//! # }
//! ```
//!
//! ### Change into continuous mode and read lux
//!
//! ```no_run
//! extern crate linux_embedded_hal as hal;
//! extern crate opt300x;
//! use opt300x::{Opt300x, SlaveAddr};
//!
//! # fn main() {
//! let dev = hal::I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::default();
//! let sensor = Opt300x::new_opt3001(dev, address);
//! let mut sensor = sensor.into_continuous().ok().unwrap();
//! loop {
//!     let lux = sensor.read_lux().unwrap();
//!     println!("lux: {:2}", lux);
//! }
//! # }
//! ```
//!
//! ### Set the integration time and manual lux range
//!
//! ```no_run
//! extern crate linux_embedded_hal as hal;
//! extern crate opt300x;
//! use opt300x::{IntegrationTime, LuxRange, Opt300x, SlaveAddr};
//!
//! # fn main() {
//! let dev = hal::I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::default();
//! let mut sensor = Opt300x::new_opt3001(dev, address);
//! sensor.set_integration_time(IntegrationTime::Ms800).unwrap();
//! sensor.set_lux_range(LuxRange::Manual(2)).unwrap();
//! sensor.enable_exponent_masking().unwrap();
//! # }
//! ```
//!
//! ### Configure interrupts
//!
//! ```no_run
//! extern crate linux_embedded_hal as hal;
//! #[macro_use]
//! extern crate nb;
//! extern crate opt300x;
//! use opt300x::{
//!     ComparisonMode, FaultCount, InterruptPinPolarity, Opt300x, SlaveAddr
//! };
//!
//! # fn main() {
//! let dev = hal::I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::default();
//! let mut sensor = Opt300x::new_opt3001(dev, address);
//! sensor.set_comparison_mode(ComparisonMode::LatchedWindow).unwrap();
//! sensor.set_interrupt_pin_polarity(InterruptPinPolarity::High).unwrap();
//! sensor.set_fault_count(FaultCount::Four).unwrap();
//! sensor.set_low_limit_raw(1, 127).unwrap();
//! sensor.set_high_limit_raw(3, 50).unwrap();
//! loop {
//!     let status = sensor.read_status().unwrap();
//!     println!("status {:?}", status);
//! }
//! # }
//! ```
//!
//! ### Enable end of conversion mode
//!
//! ```no_run
//! extern crate linux_embedded_hal as hal;
//! #[macro_use]
//! extern crate nb;
//! extern crate opt300x;
//! use opt300x::{Opt300x, SlaveAddr};
//!
//! # fn main() {
//! let dev = hal::I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::default();
//! let mut sensor = Opt300x::new_opt3001(dev, address);
//! sensor.enable_end_of_conversion_mode().unwrap();
//! loop {
//!     let status = sensor.read_status().unwrap();
//!     println!("status {:?}", status);
//! }
//! # }
//! ```

#![deny(unsafe_code, missing_docs)]
#![no_std]

use core::marker::PhantomData;
extern crate embedded_hal as hal;
extern crate nb;

/// Errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus communication error
    I2C(E),
    /// Invalid input data provided
    InvalidInputData,
}

/// Error type for mode changes.
///
/// This allows to retrieve the unchanged device in case of an error.
pub enum ModeChangeError<E, DEV> {
    /// I²C bus error while changing mode.
    ///
    /// `E` is the error that happened.
    /// `DEV` is the device with the mode unchanged.
    I2C(E, DEV),
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Config {
    bits: u16,
}

impl Config {
    fn with_high(self, mask: u16) -> Self {
        Config {
            bits: self.bits | mask,
        }
    }
    fn with_low(self, mask: u16) -> Self {
        Config {
            bits: self.bits & !mask,
        }
    }
}

/// IC markers
#[doc(hidden)]
pub mod ic {
    /// Used for OPT3001 devices
    pub struct Opt3001(());
    /// Used for OPT3002 devices
    pub struct Opt3002(());
    /// Used for OPT3004 devices
    pub struct Opt3004(());
    /// Used for OPT3006 devices
    pub struct Opt3006(());
    /// Used for OPT3007 devices
    pub struct Opt3007(());
}

/// markers
#[doc(hidden)]
pub mod marker {
    use super::private;
    pub trait WithDeviceId: private::Sealed {}
}

/// Mode marker
pub mod mode {
    /// One shot mode
    pub struct OneShot(());
    /// Continuous measurement mode
    pub struct Continuous(());
}

/// OPT300x device driver
#[derive(Debug)]
pub struct Opt300x<I2C, IC, MODE> {
    i2c: I2C,
    address: u8,
    config: Config,
    low_limit: u16,
    was_conversion_started: bool,
    _ic: PhantomData<IC>,
    _mode: PhantomData<MODE>,
}

/// Possible slave addresses
#[derive(Debug, Clone, Copy)]
pub enum SlaveAddr {
    /// Default slave address
    Default,
    /// Alternative slave address providing bit values for A1 and A0
    Alternative(bool, bool),
}

/// Fault count
///
/// Number of consecutive fault events necessary to trigger interrupt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FaultCount {
    /// One (default)
    One,
    /// Two
    Two,
    /// Four
    Four,
    /// Eight
    Eight,
}

/// Interrupt pin polarity (active state)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterruptPinPolarity {
    /// Active low (default)
    Low,
    /// Active high
    High,
}

/// Lux range
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LuxRange {
    /// Manual [0-11]
    Manual(u8),
    /// Automatic selection (default)
    Auto,
}

/// Integration time
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntegrationTime {
    /// 100 ms
    Ms100,
    /// 800 ms
    Ms800,
}

/// Result comparison mode for interrupt reporting
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComparisonMode {
    /// Latched window-style
    LatchedWindow,
    /// Transparent hysteresis-style (default)
    TransparentHysteresis,
}

/// Conversion status
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Status {
    /// Whether an overflow condition during the conversion has occurred.
    pub has_overflown: bool,
    /// Whether a new conversion is ready.
    pub conversion_ready: bool,
    /// Whether the result is higher that the configured high limit.
    pub was_too_high: bool,
    /// Whether the result is lower than the configured low limit.
    pub was_too_low: bool,
}

/// One-shot measurement
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measurement<T> {
    /// Result
    pub result: T,
    /// Conversion status.
    pub status: Status,
}

mod device_impl;
mod slave_addr;

mod private {
    use super::{ic, mode};
    pub trait Sealed {}

    impl Sealed for ic::Opt3001 {}
    impl Sealed for ic::Opt3002 {}
    impl Sealed for ic::Opt3004 {}
    impl Sealed for ic::Opt3006 {}
    impl Sealed for ic::Opt3007 {}
    impl Sealed for mode::OneShot {}
    impl Sealed for mode::Continuous {}
}
