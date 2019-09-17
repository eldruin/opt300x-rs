# Rust OPT300x Ambient Light Sensor (ALS) Driver

<!--TODO
[![crates.io](https://img.shields.io/crates/v/opt300x.svg)](https://crates.io/crates/opt300x)
[![Docs](https://docs.rs/opt300x/badge.svg)](https://docs.rs/opt300x)
-->
[![Build Status](https://travis-ci.org/eldruin/opt300x-rs.svg?branch=master)](https://travis-ci.org/eldruin/opt300x-rs)
[![Coverage Status](https://coveralls.io/repos/github/eldruin/opt300x-rs/badge.svg?branch=master)](https://coveralls.io/github/eldruin/opt300x-rs?branch=master)

This is a platform agnostic Rust driver for the OPT300x ambient light sensor
family using the [`embedded-hal`] traits.

<!--TODO
This driver allows you to:
- Enable/disable the device. See `enable()`.

[Introductory blog post](https://blog.eldruin.com/...)
-->

## The devices

This driver is compatible with the devices OPT3001, OPT3002, OPT3004, OPT3006 and OPT3007.

The OPT3001 is a sensor that measures the intensity of visible light. The spectral response of the sensor tightly matches the photopic response of the human eye and includes significant infrared rejection.

The OPT3001 is a single-chip lux meter, measuring the intensity of light as visible by the human eye. The precision spectral response and strong IR rejection of the device enables the OPT3001 to accurately meter the intensity of light as seen by the human eye regardless of light source. The strong IR rejection also aids in maintaining high accuracy when industrial design calls for mounting the sensor under dark glass for aesthetics. The OPT3001 is designed for systems that create light-based experiences for humans, and an ideal preferred replacement for photodiodes, photoresistors, or other ambient light sensors with less human eye matching and IR rejection.

Measurements can be made from 0.01 lux up to 83k lux without manually selecting full-scale ranges by using the built-in, full-scale setting feature. This capability allows light measurement over a 23-bit effective dynamic range.

The digital operation is flexible for system integration. Measurements can be either continuous or single-shot. The control and interrupt system features autonomous operation, allowing the processor to sleep while the sensor searches for appropriate wake-up events to report via the interrupt pin. The digital output is reported over an I2C- and SMBus-compatible, two-wire serial interface.

The low power consumption and low power-supply voltage capability of the OPT3001 enhance the battery life of battery-powered systems.

Datasheets:
- [OPT3001](https://www.ti.com/lit/ds/symlink/opt3001.pdf)
- [OPT3002](https://www.ti.com/lit/ds/symlink/opt3002.pdf)
- [OPT3004](https://www.ti.com/lit/ds/symlink/opt3004.pdf)
- [OPT3006](https://www.ti.com/lit/ds/symlink/opt3006.pdf)
- [OPT3007](https://www.ti.com/lit/ds/symlink/opt3007.pdf)

Application Guide:
- [OPT3001 ALS Application Guide](https://www.ti.com/lit/an/sbea002a/sbea002a.pdf)

## Usage

To use this driver, import this crate and an `embedded_hal` implementation,
then instantiate the appropriate device.

Please find additional examples using hardware in this repository: [driver-examples]

[driver-examples]: https://github.com/eldruin/driver-examples

<!--TODO
```rust
```
-->

## Status

Compatible with:
- [ ] OPT3001
- [ ] OPT3002
- [ ] OPT3004
- [ ] OPT3006
- [ ] OPT3007

## Support

For questions, issues, feature requests, and other changes, please file an
[issue in the github project](https://github.com/eldruin/opt300x-rs/issues).

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
