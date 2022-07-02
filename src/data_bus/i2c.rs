use core::{fmt, slice};

use arrayvec::ArrayVec;
use embedded_hal::blocking::i2c::{Write, WriteRead};
use num_enum::IntoPrimitive;

use super::{DataBus, MessageFlags};

#[repr(u8)]
#[derive(Clone, Copy, IntoPrimitive)]
pub enum Address {
    Default = 0x1D,
    Alt = 0x53,
}

pub struct I2CBus<I2C> {
    bus: I2C,
    address: Address,
}

impl<I2C> I2CBus<I2C> {
    pub fn new(bus: I2C, address: Address) -> Self {
        Self { bus, address }
    }
}

impl<I2C> DataBus for I2CBus<I2C>
where
    I2C: Write + WriteRead,
{
    type Error = I2CError<I2C>;
    type RawBus = I2C;

    fn read_all<R: crate::register::Register>(
        &mut self,
        buffer: &mut [u8],
    ) -> nb::Result<(), Self::Error> {
        self.bus
            .write_read(
                self.address.into(),
                &[MessageFlags::READ
                    .union(MessageFlags::MULTIPLE)
                    .register::<R>()],
                buffer,
            )
            .map_err(I2CError::WriteRead)?;
        Ok(())
    }

    fn write_all<R: crate::register::Register>(
        &mut self,
        buffer: &[u8],
    ) -> nb::Result<(), Self::Error> {
        let mut vec = ArrayVec::<_, 14>::new();
        vec.push(
            MessageFlags::WRITE
                .union(MessageFlags::MULTIPLE)
                .register::<R>(),
        );
        vec.try_extend_from_slice(buffer)
            .map_err(I2CError::Capacity)?;
        self.bus
            .write(self.address.into(), buffer)
            .map_err(I2CError::Write)?;
        Ok(())
    }

    fn read<R: crate::register::Register>(&mut self) -> nb::Result<u8, Self::Error> {
        let mut buf = 0u8;
        self.bus
            .write_read(
                self.address.into(),
                &[MessageFlags::READ
                    .union(MessageFlags::SINGLE)
                    .register::<R>()],
                slice::from_mut(&mut buf),
            )
            .map_err(I2CError::WriteRead)?;
        Ok(buf)
    }

    fn write<R: crate::register::Register>(&mut self, data: u8) -> nb::Result<(), Self::Error> {
        let msg = [
            MessageFlags::WRITE
                .union(MessageFlags::SINGLE)
                .register::<R>(),
            data,
        ];
        self.bus
            .write(self.address.into(), &msg)
            .map_err(I2CError::Write)?;
        Ok(())
    }

    fn destroy(self) -> Self::RawBus {
        self.bus
    }
}

pub enum I2CError<I2C>
where
    I2C: Write + WriteRead,
{
    WriteRead(<I2C as WriteRead>::Error),
    Write(<I2C as Write>::Error),
    Capacity(arrayvec::CapacityError),
}

impl<I2C> fmt::Debug for I2CError<I2C>
where
    I2C: Write + WriteRead,
    <I2C as WriteRead>::Error: fmt::Debug,
    <I2C as Write>::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            I2CError::WriteRead(error) => write!(f, "WriteRead({error:?})"),
            I2CError::Write(error) => write!(f, "Write({error:?})"),
            I2CError::Capacity(error) => write!(f, "Capacity({error:?})"),
        }
    }
}
