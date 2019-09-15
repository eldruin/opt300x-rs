//! This is a platform agnostic Rust driver for the OPT300x ambient light
//! sensors using the [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
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

#![deny(unsafe_code, missing_docs)]
#![no_std]

