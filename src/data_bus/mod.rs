pub mod i2c;
mod spi;

pub use i2c::I2CBus;
pub use spi::SPIBus;

use bitflags::bitflags;

use crate::register::Register;

pub trait DataBus {
    type Error;
    type RawBus;
    fn read_all<R: Register>(&mut self, buffer: &mut [u8]) -> nb::Result<(), Self::Error>;
    fn write_all<R: Register>(&mut self, buffer: &[u8]) -> nb::Result<(), Self::Error>;
    fn read<R: Register>(&mut self) -> nb::Result<u8, Self::Error>;
    fn write<R: Register>(&mut self, data: u8) -> nb::Result<(), Self::Error>;

    fn destroy(self) -> Self::RawBus;
}

bitflags! {
    struct MessageFlags: u8 {
        const READ = 0b1011_1111;
        const WRITE = 0b0011_1111;
        const SINGLE = 0b0011_1111;
        const MULTIPLE = 0b0111_1111;
    }
}

impl MessageFlags {
    pub fn register<R: Register>(self) -> u8 {
        self.bits & R::ADDRESS
    }
}
