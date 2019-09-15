use hal::blocking::i2c;
use {ic, Config, Error, FaultCount, InterruptPinPolarity, Opt300x, PhantomData, SlaveAddr};

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
    const OVF: u16 = 1 << 8;
    const POL: u16 = 1 << 3;
}

impl<I2C, E> Opt300x<I2C, ic::Opt3001>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    /// Create new instance of the OPT3001 device.
    pub fn new_opt3001(i2c: I2C, address: SlaveAddr) -> Self {
        Opt300x {
            i2c,
            address: address.addr(),
            config: Config { bits: 0xC810 },
            _ic: PhantomData,
        }
    }
}

impl<I2C, E, IC> Opt300x<I2C, IC>
where
    I2C: i2c::WriteRead<Error = E> + i2c::Write<Error = E>,
{
    /// Destroy driver instance, return I²C bus instance.
    pub fn destroy(self) -> I2C {
        self.i2c
    }

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

    /// Read the manifacturer ID
    pub fn get_manufacturer_id(&mut self) -> Result<u16, Error<E>> {
        self.read_register(Register::MANUFACTURER_ID)
    }

    /// Read the device ID
    pub fn get_device_id(&mut self) -> Result<u16, Error<E>> {
        self.read_register(Register::DEVICE_ID)
    }

    fn read_register(&mut self, register: u8) -> Result<u16, Error<E>> {
        let mut data = [0, 0];
        self.i2c
            .write_read(self.address, &[register], &mut data)
            .map_err(Error::I2C)
            .and(Ok(u16::from(data[0]) << 8 | u16::from(data[1])))
    }

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
