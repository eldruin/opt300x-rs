use crate::hal::blocking::i2c;
use crate::{
    ic, mode, ComparisonMode, Config, Error, FaultCount, IntegrationTime, InterruptPinPolarity,
    LuxRange, ModeChangeError, Opt300x, PhantomData, SlaveAddr,
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
    const L: u16 = 1 << 4;
    const POL: u16 = 1 << 3;
    const ME: u16 = 1 << 2;
}

impl<I2C> Opt300x<I2C, ic::Opt3001, mode::OneShot> {
    /// Create new instance of the OPT3001 device.
    pub fn new_opt3001(i2c: I2C, address: SlaveAddr) -> Self {
        Opt300x {
            i2c,
            address: address.addr(),
            config: Config { bits: 0xC810 },
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
        let result = self.read_register(Register::RESULT)?;
        let exp = result >> 12;
        let mantissa = result & 0xFFF;
        Ok((f64::from(1 << exp) * 0.01 * f64::from(mantissa)) as f32)
    }

    /// Read the result of the most recent light to digital conversion in
    /// raw format: (exponent, mantissa)
    pub fn read_raw(&mut self) -> Result<(u8, u16), Error<E>> {
        let result = self.read_register(Register::RESULT)?;
        Ok(((result >> 12) as u8, result & 0xFFF))
    }
}

impl<I2C, E, IC, MODE> Opt300x<I2C, IC, MODE>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    /// Read whether an overflow condition has occurred
    pub fn has_overflown(&mut self) -> Result<bool, Error<E>> {
        Ok((self.read_register(Register::CONFIG)? & BitFlags::OVF) != 0)
    }

    /// Set the fault count
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
    pub fn set_integration_time(&mut self, time: IntegrationTime) -> Result<(), Error<E>> {
        let config = match time {
            IntegrationTime::Ms100 => self.config.with_low(BitFlags::CT),
            IntegrationTime::Ms800 => self.config.with_high(BitFlags::CT),
        };
        self.set_config(config)
    }

    /// Set the interrupt pin polarity
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
    pub fn enable_exponent_masking(&mut self) -> Result<(), Error<E>> {
        self.set_config(self.config.with_high(BitFlags::ME))
    }

    /// Disable exponent masking (default).
    pub fn disable_exponent_masking(&mut self) -> Result<(), Error<E>> {
        self.set_config(self.config.with_low(BitFlags::ME))
    }

    /// Set result comparison mode for interrupt reporting
    pub fn set_comparison_mode(&mut self, mode: ComparisonMode) -> Result<(), Error<E>> {
        let config = match mode {
            ComparisonMode::LatchedWindow => self.config.with_high(BitFlags::L),
            ComparisonMode::TransparentHysteresis => self.config.with_low(BitFlags::L),
        };
        self.set_config(config)
    }

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
        let data = [
            Register::CONFIG,
            (config.bits >> 8) as u8,
            config.bits as u8,
        ];
        self.i2c.write(self.address, &data).map_err(Error::I2C)?;
        self.config = config;
        Ok(())
    }
}
