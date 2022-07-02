use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::private::Sealed;

pub(super) trait RegisterField: Sealed + Sized {
    fn from_byte(value: u8) -> Self;

    fn to_byte(self) -> u8;

    fn from_inner_bits(byte: u8, msb: u8, lsb: u8) -> Self {
        let offset = msb - lsb;
        Self::from_byte(byte << (7 - msb) >> (7 - offset))
    }

    fn set_inner_bits(byte: u8, msb: u8, lsb: u8, value: Self) -> u8 {
        let offset = msb - lsb;
        let mask = (0xFFu8 << (7 - msb) >> (7 - offset) << lsb) ^ 0xFF;
        let value = value.to_byte() << (7 - offset) >> (7 - msb);

        (byte & mask) | value
    }
}

impl Sealed for bool {}

impl RegisterField for bool {
    #[inline]
    fn from_byte(value: u8) -> Self {
        value > 0
    }

    #[inline]
    fn to_byte(self) -> u8 {
        self.into()
    }

    #[inline]
    fn from_inner_bits(byte: u8, msb: u8, _lsb: u8) -> Self {
        Self::from_byte(byte << (7 - msb) >> 7)
    }

    #[inline]
    fn set_inner_bits(byte: u8, msb: u8, _lsb: u8, value: Self) -> u8 {
        let mask = 0b1111_1110u8.rotate_left(msb as u32);
        (byte & mask) | (u8::from(value) << msb)
    }
}

impl Sealed for i8 {}

impl RegisterField for i8 {
    fn from_byte(value: u8) -> Self {
        value as i8
    }

    fn to_byte(self) -> u8 {
        self as u8
    }
}

impl Sealed for u8 {}

impl RegisterField for u8 {
    fn from_byte(value: u8) -> Self {
        value
    }

    fn to_byte(self) -> u8 {
        self
    }
}

#[repr(u8)]
#[derive(Clone, Copy, TryFromPrimitive, IntoPrimitive)]
pub enum FIFOMode {
    Bypass = 0,
    Fifo = 1,
    Stream = 2,
    Trigger = 3,
}

impl Sealed for FIFOMode {}

impl RegisterField for FIFOMode {
    fn from_byte(value: u8) -> Self {
        FIFOMode::try_from(value).unwrap_or_else(|_| panic!())
    }

    fn to_byte(self) -> u8 {
        u8::from(self)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, TryFromPrimitive, IntoPrimitive)]
pub enum GRange {
    Two = 0,
    Four = 1,
    Eight = 2,
    Sixteen = 3,
}

impl Sealed for GRange {}

impl RegisterField for GRange {
    fn from_byte(value: u8) -> Self {
        GRange::try_from(value).unwrap_or_else(|_| panic!())
    }

    fn to_byte(self) -> u8 {
        u8::from(self)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, TryFromPrimitive, IntoPrimitive)]
pub enum ReadingFrequencyHz {
    Eight = 0,
    Four = 1,
    Two = 2,
    One = 3,
}

impl Sealed for ReadingFrequencyHz {}

impl RegisterField for ReadingFrequencyHz {
    fn from_byte(value: u8) -> Self {
        ReadingFrequencyHz::try_from(value).unwrap_or_else(|_| panic!())
    }

    fn to_byte(self) -> u8 {
        u8::from(self)
    }
}

/// Represents all the possible output data rates for the device.
/// Any underscore after a number represents a decimal dot.
///
/// For example, `_0_78` represents a ***0.78 Hz*** rate, and `_400` represents
/// a ***400 Hz*** rate.
#[repr(u8)]
#[derive(Clone, Copy, TryFromPrimitive, IntoPrimitive)]
pub enum OutputDataRateHz {
    _0_10 = 0,
    _0_20 = 1,
    _0_39 = 2,
    _0_78 = 3,
    _1_56 = 4,
    _3_13 = 5,
    _6_25 = 6,
    _12_5 = 7,
    _25 = 8,
    _50 = 9,
    _100 = 10,
    _200 = 11,
    _400 = 12,
    _800 = 13,
    _1600 = 14,
    _3200 = 15,
}

impl Sealed for OutputDataRateHz {}

impl RegisterField for OutputDataRateHz {
    fn from_byte(value: u8) -> Self {
        OutputDataRateHz::try_from(value).unwrap_or_else(|_| panic!())
    }

    fn to_byte(self) -> u8 {
        u8::from(self)
    }
}

#[cfg(test)]
mod tests {

    use super::RegisterField;

    #[test]
    fn bit_mangling() {
        let input = 0b0100_1100;

        assert!(bool::from_inner_bits(input, 3, 3));
        assert!(!bool::from_inner_bits(input, 5, 5));

        assert_eq!(u8::from_inner_bits(input, 6, 2), 0b10011);

        assert_eq!(bool::set_inner_bits(input, 5, 5, true), 0b0110_1100);
        assert_eq!(bool::set_inner_bits(input, 6, 6, false), 0b0000_1100);

        assert_eq!(u8::set_inner_bits(input, 6, 3, 0b0000_1010), 0b0101_0100);
        assert_eq!(u8::set_inner_bits(input, 6, 3, 0b1101_1010), 0b0101_0100);
    }
}
