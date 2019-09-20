use crate::hal::blocking::i2c;
use crate::{
    ic, mode, ComparisonMode, Config, Error, FaultCount, IntegrationTime, InterruptPinPolarity,
    LuxRange, Measurement, ModeChangeError, Opt300x, PhantomData, SlaveAddr, Status,
};

struct Register;
impl Register {
    const RESULT: u8 = 0x00;
    const CONFIG: u8 = 0x01;
    const LOW_LIMIT: u8 = 0x02;
    const HIGH_LIMIT: u8 = 0x03;
    const MANUFACTURER_ID: u8 = 0x7E;
    const DEVICE_ID: u8 = 0x7F;
}

struct BitFlags;
impl BitFlags {
    const CT: u16 = 1 << 11;
    const MODE1: u16 = 1 << 10;
    const MODE0: u16 = 1 << 9;
    const OVF: u16 = 1 << 8;
    const CRF: u16 = 1 << 7;
    const FH: u16 = 1 << 6;
    const FL: u16 = 1 << 5;
    const L: u16 = 1 << 4;
    const POL: u16 = 1 << 3;
    const ME: u16 = 1 << 2;
}

impl Default for Config {
    fn default() -> Self {
        Config { bits: 0xC810 }
    }
}

impl<I2C> Opt300x<I2C, ic::Opt3001, mode::OneShot> {
    /// Create new instance of the OPT3001 device.
    pub fn new_opt3001(i2c: I2C, address: SlaveAddr) -> Self {
        Opt300x {
            i2c,
            address: address.addr(),
            config: Config::default(),
            low_limit: 0,
            was_conversion_started: false,
            _ic: PhantomData,
            _mode: PhantomData,
        }
    }
}

impl<I2C, IC, MODE> Opt300x<I2C, IC, MODE> {
    /// Destroy driver instance, return I²C bus instance.
    pub fn destroy(self) -> I2C {
        self.i2c
    }
}

impl<I2C, E, IC> Opt300x<I2C, IC, mode::OneShot>
where
    I2C: i2c::Write<Error = E>,
{
    /// Change into continuous measurement mode
    ///
    /// Note that the conversion ready flag is cleared automatically
    /// after calling this method.
    pub fn into_continuous(
        mut self,
    ) -> Result<Opt300x<I2C, IC, mode::Continuous>, ModeChangeError<E, Self>> {
        if let Err(Error::I2C(e)) = self.set_config(
            self.config
                .with_high(BitFlags::MODE0)
                .with_high(BitFlags::MODE1),
        ) {
            return Err(ModeChangeError::I2C(e, self));
        }
        Ok(Opt300x {
            i2c: self.i2c,
            address: self.address,
            config: self.config,
            low_limit: self.low_limit,
            was_conversion_started: false,
            _ic: PhantomData,
            _mode: PhantomData,
        })
    }
}

impl<I2C, E, IC> Opt300x<I2C, IC, mode::Continuous>
where
    I2C: i2c::Write<Error = E>,
{
    /// Change into one-shot mode
    ///
    /// This will actually shut down the device until a measurement is requested.
    pub fn into_one_shot(
        mut self,
    ) -> Result<Opt300x<I2C, IC, mode::OneShot>, ModeChangeError<E, Self>> {
        if let Err(Error::I2C(e)) = self.set_config(
            self.config
                .with_low(BitFlags::MODE0)
                .with_low(BitFlags::MODE1),
        ) {
            return Err(ModeChangeError::I2C(e, self));
        }
        Ok(Opt300x {
            i2c: self.i2c,
            address: self.address,
            config: self.config,
            low_limit: self.low_limit,
            was_conversion_started: false,
            _ic: PhantomData,
            _mode: PhantomData,
        })
    }
}

impl<I2C, E, IC> Opt300x<I2C, IC, mode::Continuous>
where
    I2C: i2c::WriteRead<Error = E>,
{
    /// Read the result of the most recent light to digital conversion in lux
    pub fn read_lux(&mut self) -> Result<f32, Error<E>> {
        let result = self.read_raw()?;
        Ok(raw_to_lux(result))
    }

    /// Read the result of the most recent light to digital conversion in
    /// raw format: (exponent, mantissa)
    pub fn read_raw(&mut self) -> Result<(u8, u16), Error<E>> {
        let result = self.read_register(Register::RESULT)?;
        Ok(((result >> 12) as u8, result & 0xFFF))
    }
}

fn raw_to_lux(result: (u8, u16)) -> f32 {
    (f64::from(1 << result.0) * 0.01 * f64::from(result.1)) as f32
}

impl<I2C, E, IC> Opt300x<I2C, IC, mode::OneShot>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    /// Read the result of the most recent light to digital conversion in lux
    pub fn read_lux(&mut self) -> nb::Result<Measurement<f32>, Error<E>> {
        let measurement = self.read_raw()?;
        Ok(Measurement {
            result: raw_to_lux(measurement.result),
            status: measurement.status,
        })
    }

    /// Read the result of the most recent light to digital conversion in
    /// raw format: (exponent, mantissa)
    pub fn read_raw(&mut self) -> nb::Result<Measurement<(u8, u16)>, Error<E>> {
        if self.was_conversion_started {
            let status = self.read_status().map_err(nb::Error::Other)?;
            if status.conversion_ready {
                let result = self
                    .read_register(Register::RESULT)
                    .map_err(nb::Error::Other)?;
                self.was_conversion_started = false;
                Ok(Measurement {
                    result: ((result >> 12) as u8, result & 0xFFF),
                    status,
                })
            } else {
                Err(nb::Error::WouldBlock)
            }
        } else {
            let config = self.config.with_high(BitFlags::MODE0);
            self.write_register(Register::CONFIG, config.bits)
                .map_err(nb::Error::Other)?;
            self.was_conversion_started = true;
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<I2C, E, IC, MODE> Opt300x<I2C, IC, MODE>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    /// Read the status of the conversion.
    ///
    /// Note that the conversion ready flag is cleared automatically
    /// after calling this method.
    pub fn read_status(&mut self) -> Result<Status, Error<E>> {
        let config = self.read_register(Register::CONFIG)?;
        Ok(Status {
            has_overflown: (config & BitFlags::OVF) != 0,
            conversion_ready: (config & BitFlags::CRF) != 0,
            was_too_high: (config & BitFlags::FH) != 0,
            was_too_low: (config & BitFlags::FL) != 0,
        })
    }
}

impl<I2C, E, IC, MODE> Opt300x<I2C, IC, MODE>
where
    I2C: i2c::Write<Error = E>,
{
    /// Set the fault count
    ///
    /// Note that the conversion ready flag is cleared automatically
    /// after calling this method.
    pub fn set_fault_count(&mut self, count: FaultCount) -> Result<(), Error<E>> {
        let config = self.config.bits & !0b11;
        let config = match count {
            FaultCount::One => config,
            FaultCount::Two => config | 0b01,
            FaultCount::Four => config | 0b10,
            FaultCount::Eight => config | 0b11,
        };
        self.set_config(Config { bits: config })
    }

    /// Set the lux range.
    ///
    /// `Error::InvalidInputData` will be returned for manual values outside
    /// the valid range.
    ///
    /// Note that the conversion ready flag is cleared automatically
    /// after calling this method.
    pub fn set_lux_range(&mut self, range: LuxRange) -> Result<(), Error<E>> {
        let value = match range {
            LuxRange::Auto => Ok(0b1100),
            LuxRange::Manual(rn) if rn >= 0b1100 => Err(Error::InvalidInputData),
            LuxRange::Manual(rn) => Ok(rn),
        }?;
        let config = self.config.bits & 0x0FFF;
        self.set_config(Config {
            bits: config | (u16::from(value) << 12),
        })
    }

    /// Set the integration (conversion) time.
    ///
    /// Note that the conversion ready flag is cleared automatically
    /// after calling this method.
    pub fn set_integration_time(&mut self, time: IntegrationTime) -> Result<(), Error<E>> {
        let config = match time {
            IntegrationTime::Ms100 => self.config.with_low(BitFlags::CT),
            IntegrationTime::Ms800 => self.config.with_high(BitFlags::CT),
        };
        self.set_config(config)
    }

    /// Set the interrupt pin polarity
    ///
    /// Note that the conversion ready flag is cleared automatically
    /// after calling this method.
    pub fn set_interrupt_pin_polarity(
        &mut self,
        polarity: InterruptPinPolarity,
    ) -> Result<(), Error<E>> {
        let config = match polarity {
            InterruptPinPolarity::Low => self.config.with_low(BitFlags::POL),
            InterruptPinPolarity::High => self.config.with_high(BitFlags::POL),
        };
        self.set_config(config)
    }

    /// Enable exponent masking.
    ///
    /// Note that the conversion ready flag is cleared automatically
    /// after calling this method.
    pub fn enable_exponent_masking(&mut self) -> Result<(), Error<E>> {
        self.set_config(self.config.with_high(BitFlags::ME))
    }

    /// Disable exponent masking (default).
    ///
    /// Note that the conversion ready flag is cleared automatically
    /// after calling this method.
    pub fn disable_exponent_masking(&mut self) -> Result<(), Error<E>> {
        self.set_config(self.config.with_low(BitFlags::ME))
    }

    /// Set result comparison mode for interrupt reporting
    ///
    /// Note that the conversion ready flag is cleared automatically
    /// after calling this method.
    pub fn set_comparison_mode(&mut self, mode: ComparisonMode) -> Result<(), Error<E>> {
        let config = match mode {
            ComparisonMode::LatchedWindow => self.config.with_high(BitFlags::L),
            ComparisonMode::TransparentHysteresis => self.config.with_low(BitFlags::L),
        };
        self.set_config(config)
    }

    /// Set the lux low limit in raw format (exponent, mantissa).
    ///
    /// Returns `Error::InvalidInputData` for an exponent value greater than
    /// 11 or a mantissa value greater than 4095.
    ///
    /// Note that this disables the end-of-conversion mode.
    pub fn set_low_limit_raw(&mut self, exponent: u8, mantissa: u16) -> Result<(), Error<E>> {
        if exponent > 0b1011 || mantissa > 0xFFF {
            return Err(Error::InvalidInputData);
        }
        let limit = u16::from(exponent) << 12 | mantissa;
        self.write_register(Register::LOW_LIMIT, limit)?;
        self.low_limit = limit;
        Ok(())
    }

    /// Set the lux high limit in raw format (exponent, mantissa).
    ///
    /// Returns `Error::InvalidInputData` for an exponent value greater than
    /// 11 or a mantissa value greater than 4095.
    pub fn set_high_limit_raw(&mut self, exponent: u8, mantissa: u16) -> Result<(), Error<E>> {
        if exponent > 0b1011 || mantissa > 0xFFF {
            return Err(Error::InvalidInputData);
        }
        let limit = u16::from(exponent) << 12 | mantissa;
        self.write_register(Register::HIGH_LIMIT, limit)
    }

    /// Enable end-of-conversion mode
    ///
    /// Note that this changes the two highest bits of the lux low limit exponent.
    /// Please see the device datasheet for further details.
    pub fn enable_end_of_conversion_mode(&mut self) -> Result<(), Error<E>> {
        let limit = self.low_limit | 0b1100 << 12;
        self.write_register(Register::LOW_LIMIT, limit)
    }

    /// Disable end-of-conversion mode
    ///
    /// Note that this restores the two highest bits of the lux low limit
    /// exponent to the last value set before enabling the end-of-conversion
    /// mode (0b00 by default).
    pub fn disable_end_of_conversion_mode(&mut self) -> Result<(), Error<E>> {
        self.write_register(Register::LOW_LIMIT, self.low_limit)
    }
}

impl<I2C, E, IC, MODE> Opt300x<I2C, IC, MODE>
where
    I2C: i2c::WriteRead<Error = E>,
{
    /// Read the manifacturer ID
    pub fn get_manufacturer_id(&mut self) -> Result<u16, Error<E>> {
        self.read_register(Register::MANUFACTURER_ID)
    }

    /// Read the device ID
    pub fn get_device_id(&mut self) -> Result<u16, Error<E>> {
        self.read_register(Register::DEVICE_ID)
    }
}

impl<I2C, E, IC, MODE> Opt300x<I2C, IC, MODE>
where
    I2C: i2c::WriteRead<Error = E>,
{
    fn read_register(&mut self, register: u8) -> Result<u16, Error<E>> {
        let mut data = [0, 0];
        self.i2c
            .write_read(self.address, &[register], &mut data)
            .map_err(Error::I2C)
            .and(Ok(u16::from(data[0]) << 8 | u16::from(data[1])))
    }
}

impl<I2C, E, IC, MODE> Opt300x<I2C, IC, MODE>
where
    I2C: i2c::Write<Error = E>,
{
    fn set_config(&mut self, config: Config) -> Result<(), Error<E>> {
        self.write_register(Register::CONFIG, config.bits)?;
        self.config = config;
        Ok(())
    }

    fn write_register(&mut self, register: u8, value: u16) -> Result<(), Error<E>> {
        let data = [register, (value >> 8) as u8, value as u8];
        self.i2c.write(self.address, &data).map_err(Error::I2C)
    }
}
