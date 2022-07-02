use core::fmt;

use embedded_hal::{digital::v2::OutputPin, spi};
use nb::block;

use crate::register::Register;

use super::{DataBus, MessageFlags};
pub use raw::SPIBus;

mod raw {
    use core::{
        ops::{Deref, DerefMut},
        panic, marker::PhantomData,
    };
    use embedded_hal::{digital::v2::OutputPin, spi};
    use nb::block;

    use super::SpiError;
    pub struct SPIBus<SPI, CS: OutputPin, Word> {
        bus: SPI,
        cs: CS,
        _word: PhantomData<Word>
    }

    impl<SPI, CS, Word> SPIBus<SPI, CS, Word>
    where
        SPI: spi::FullDuplex<Word>,
        CS: OutputPin,
    {
        pub fn new(bus: SPI, cs: CS) -> Self {
            Self { bus, cs, _word: PhantomData }
        }
        pub(super) fn spi_handle(&mut self) -> Result<SPIBusGuard<SPI, CS, Word>, SpiError<SPI, CS, Word>> {
            SPIBusGuard::new(self)
        }

        pub(super) fn destroy(self) -> (SPI, CS) {
            (self.bus, self.cs)
        }
    }
    pub(super) struct SPIBusGuard<'a, SPI, CS: OutputPin, Word>(&'a mut SPIBus<SPI, CS, Word>);

    impl<'a, SPI, CS, Word> SPIBusGuard<'a, SPI, CS, Word>
    where
        SPI: spi::FullDuplex<Word>,
        CS: OutputPin,
    {
        fn new(bus: &'a mut SPIBus<SPI, CS, Word>) -> Result<Self, SpiError<SPI, CS, Word>> {
            bus.cs
                .set_low()
                .map(|_| Self(bus))
                .map_err(SpiError::ChipSelect)
        }

        pub fn exchange(&mut self, word: Word) -> nb::Result<Word, SpiError<SPI, CS, Word>> {
            self.0
                .bus
                .send(word)
                .map_err(|e| e.map(SpiError::Transfer))?;
            Ok(block!(self.0.bus.read()).map_err(SpiError::Read)?)
        }
    }

    impl<'a, SPI, CS: OutputPin, Word> Deref for SPIBusGuard<'a, SPI, CS, Word> {
        type Target = SPI;
        fn deref(&self) -> &Self::Target {
            &self.0.bus
        }
    }

    impl<'a, SPI, CS: OutputPin, Word> DerefMut for SPIBusGuard<'a, SPI, CS, Word> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0.bus
        }
    }

    impl<'a, SPI, CS: OutputPin, Word> Drop for SPIBusGuard<'a, SPI, CS, Word> {
        fn drop(&mut self) {
            self.0.cs.set_high().unwrap_or_else(|_| panic!());
        }
    }
}

impl<SPI, CS> DataBus for raw::SPIBus<SPI, CS, u16>
where
    SPI: spi::FullDuplex<u16>,
    CS: OutputPin,
{
    type Error = SpiError<SPI, CS, u16>;
    type RawBus = (SPI, CS);
    fn read_all<R: Register>(&mut self, buffer: &mut [u8]) -> nb::Result<(), Self::Error> {
        let (first, elements) = if let Some((first, elements)) = buffer.split_first_mut() {
            (first, elements)
        } else {
            return Ok(());
        };

        let msg = u16::from_be_bytes([
            MessageFlags::READ
                .union(MessageFlags::MULTIPLE)
                .register::<R>(),
            0,
        ]);

        let mut spi = self.spi_handle()?;

        *first = spi.exchange(msg).map(|word| word.to_be_bytes()[1])?;

        let mut chunked = elements.chunks_exact_mut(2);
        for chunk in &mut chunked {
            let result = block!(spi.exchange(0)).map(|word| word.to_be_bytes())?;
            chunk.copy_from_slice(&result);
        }

        if let [last] = chunked.into_remainder() {
            *last = block!(spi.exchange(0)).map(|word| word.to_be_bytes()[0])?;
        }
        Ok(())
    }

    fn write_all<R: Register>(&mut self, buffer: &[u8]) -> nb::Result<(), Self::Error> {
        if buffer.len() > 1 && buffer.len() % 2 == 0 {
            return Err(SpiError::InvalidWriteBuffer("Buffer length must be 0, 1 or odd.").into());
        }
        let (first, elements) = if let Some((first, elements)) = buffer.split_first() {
            (first, elements)
        } else {
            return Ok(());
        };

        let msg = u16::from_be_bytes([
            MessageFlags::WRITE
                .union(MessageFlags::MULTIPLE)
                .register::<R>(),
            *first,
        ]);

        let mut spi = self.spi_handle()?;

        spi.exchange(msg)?;

        let mut chunked = elements.chunks_exact(2);
        for chunk in &mut chunked {
            let word = u16::from_be_bytes([chunk[0], chunk[1]]);
            block!(spi.exchange(word))?;
        }

        Ok(())
    }

    fn read<R: Register>(&mut self) -> nb::Result<u8, Self::Error> {
        let msg = u16::from_be_bytes([
            MessageFlags::READ
                .union(MessageFlags::SINGLE)
                .register::<R>(),
            0,
        ]);

        let mut spi = self.spi_handle()?;

        let output = spi.exchange(msg)?;
        Ok(output as u8)
    }

    fn write<R: Register>(&mut self, data: u8) -> nb::Result<(), Self::Error> {
        let msg = u16::from_be_bytes([
            MessageFlags::WRITE
                .union(MessageFlags::SINGLE)
                .register::<R>(),
            data,
        ]);

        let mut spi = self.spi_handle()?;

        spi.exchange(msg)?;

        Ok(())
    }

    fn destroy(self) -> Self::RawBus {
        self.destroy()
    }
}

pub enum SpiError<SPI, CS, Word>
where
    SPI: spi::FullDuplex<Word>,
    CS: OutputPin,
{
    /// SPI error occured during a read transaction
    Read(<SPI as spi::FullDuplex<Word>>::Error),

    /// SPI error occured during a transfer transaction
    Transfer(<SPI as spi::FullDuplex<Word>>::Error),

    /// SPI error occured during a transfer transaction
    InvalidWriteBuffer(&'static str),

    /// Error occured while changing chip select signal
    ChipSelect(<CS as OutputPin>::Error),
}

impl<SPI, CS, Word> fmt::Debug for SpiError<SPI, CS, Word>
where
    SPI: spi::FullDuplex<Word>,
    CS: OutputPin,
    <SPI as spi::FullDuplex<Word>>::Error: fmt::Debug,
    <CS as OutputPin>::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpiError::Read(error) => write!(f, "Read({error:?})"),
            SpiError::Transfer(error) => write!(f, "Transfer({error:?})"),
            SpiError::ChipSelect(error) => write!(f, "ChipSelect({error:?})"),
            SpiError::InvalidWriteBuffer(error) => write!(f, "InvalidWriteBuffer({error:?})"),
        }
    }
}
