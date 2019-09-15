//! Slave address implementation
use SlaveAddr;

const DEVICE_BASE_ADDRESS: u8 = 0b100_0100;

impl Default for SlaveAddr {
    /// Default slave address
    fn default() -> Self {
        SlaveAddr::Default
    }
}

impl SlaveAddr {
    /// Get slave address
    pub(crate) fn addr(self) -> u8 {
        match self {
            SlaveAddr::Default => DEVICE_BASE_ADDRESS,
            SlaveAddr::Alternative(a1, a0) => {
                SlaveAddr::default().addr() | ((a1 as u8) << 1) | a0 as u8
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate embedded_hal_mock as hal;
    use super::*;

    #[test]
    fn default_address_is_correct() {
        assert_eq!(DEVICE_BASE_ADDRESS, SlaveAddr::default().addr());
    }

    #[test]
    fn can_generate_alternative_addresses() {
        const ADDR: u8 = DEVICE_BASE_ADDRESS;
        assert_eq!(ADDR, SlaveAddr::Alternative(false, false).addr());
        assert_eq!(ADDR | 0b01, SlaveAddr::Alternative(false, true).addr());
        assert_eq!(ADDR | 0b10, SlaveAddr::Alternative(true, false).addr());
        assert_eq!(ADDR | 0b11, SlaveAddr::Alternative(true, true).addr());
    }
}
