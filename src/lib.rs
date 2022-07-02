#![cfg_attr(not(test), no_std)]

use data_bus::DataBus;
use embedded_hal::{blocking::i2c, digital::v2::OutputPin, spi};

pub mod data_bus;
pub mod register;

pub struct ADXL345<BUS> {
    bus: BUS,
}

impl<BUS: DataBus> ADXL345<BUS> {
    pub fn destroy(self) -> BUS::RawBus {
        self.bus.destroy()
    }
}

impl<SPI, CS> ADXL345<data_bus::SPIBus<SPI, CS, u16>>
where
    SPI: spi::FullDuplex<u16>,
    CS: OutputPin,
{
    pub fn from_spi_cs(bus: SPI, cs: CS) -> Self {
        Self {
            bus: data_bus::SPIBus::new(bus, cs),
        }
    }
}

impl<I2C> ADXL345<data_bus::I2CBus<I2C>>
where
    I2C: i2c::Write + i2c::WriteRead,
{
    pub fn from_i2c(bus: I2C, address: data_bus::i2c::Address) -> Self {
        Self {
            bus: data_bus::I2CBus::new(bus, address),
        }
    }
}

mod private {
    /// Super trait used to mark traits with an exhaustive set of
    /// implementations
    pub trait Sealed {}
}
