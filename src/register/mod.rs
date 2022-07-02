mod field;

use core::marker::PhantomData;

use paste::paste;

pub use field::*;

use crate::{data_bus::DataBus, private, ADXL345};

/// This trait is sealed to disallow external implementations.
pub trait Register: private::Sealed {
    const ADDRESS: u8;
}

/// This trait is sealed to disallow external implementations.
pub trait Readable: private::Sealed {
    type Handle;

    #[doc(hidden)]
    fn fill(data: u8) -> Self::Handle;
}

/// This trait is sealed to disallow external implementations.
pub trait Writable: Readable + private::Sealed {
    fn into_raw(w: Self::Handle) -> u8;
}

pub struct RegisterHandle<'s, R, BUS>(&'s mut ADXL345<BUS>, PhantomData<R>);

impl<'s, R, BUS> RegisterHandle<'s, R, BUS>
where
    BUS: DataBus,
    R: Register,
{
    /// Read the specified register
    pub fn read(&mut self) -> nb::Result<R::Handle, BUS::Error>
    where
        R: Readable,
    {
        let reg = self.0.bus.read::<R>()?;

        Ok(R::fill(reg))
    }

    /// Write to the specified register
    pub fn write<F>(&mut self, f: F) -> nb::Result<(), BUS::Error>
    where
        R: Writable,
        R::Handle: Default,
        F: FnOnce(&mut R::Handle) -> &mut R::Handle,
    {
        let mut reg = R::Handle::default();

        f(&mut reg);

        self.0.bus.write::<R>(R::into_raw(reg))?;

        Ok(())
    }

    /// Modify the specified register
    pub fn modify<F>(&mut self, f: F) -> nb::Result<(), BUS::Error>
    where
        R: Writable,
        F: FnOnce(&mut R::Handle) -> &mut R::Handle,
    {
        let mut reg = self.read()?;

        f(&mut reg);

        self.0.bus.write::<R>(R::into_raw(reg))?;

        Ok(())
    }
}

macro_rules! sys_register {
    (
        $(
            $addr:expr,
            $rw:tt,
            $name:ident {
            #[$doc:meta]
            $(
                $field:ident,
                $msb:expr,
                $lsb:expr,
                $ty:ty;
                #[$field_doc:meta]
            )*
            }
        )*
    ) => {
        paste! {
            $(
                #[$doc]
                #[allow(non_camel_case_types)]
                pub struct $name;

                impl Register for $name {
                    const ADDRESS: u8 = $addr;
                }

                #[$doc]
                pub mod [<$name:lower>] {
                    #[allow(unused_imports)]
                    use super::*;

                    #[doc = "Handle to manipulate the `" $name "` register"]
                    pub struct Handle(pub(super) u8);

                    impl_fields_rw!(
                        $rw,
                        $(
                            $field,
                            $msb,
                            $lsb,
                            $ty;
                            #[$field_doc]
                        )*
                    );
                }

                impl private::Sealed for $name {}

                impl_rw!($rw, $name);
            )*

            impl<BUS> crate::ADXL345<BUS> {
                $(
                    #[$doc]
                    pub fn [<$name:lower>](&mut self) -> RegisterHandle<$name, BUS> {
                        RegisterHandle(self, PhantomData)
                    }
                )*
            }
        }
    };
}

macro_rules! impl_fields_rw {
    (
        RO,
        $(
            $field:ident,
            $msb:expr,
            $lsb:expr,
            $ty:ty;
            #[$field_doc:meta]
        )*
    ) => {
        impl_fields_rw!(
            @R,
            $(
                $field,
                $msb,
                $lsb,
                $ty;
                #[$field_doc]
            )*
        );
    };
    (
        RW,
        $(
            $field:ident,
            $msb:expr,
            $lsb:expr,
            $ty:ty;
            #[$field_doc:meta]
        )*
    ) => {
        impl_fields_rw!(
            @R,
            $(
                $field,
                $msb,
                $lsb,
                $ty;
                #[$field_doc]
            )*
        );
        impl_fields_rw!(
            @W,
            $(
                $field,
                $msb,
                $lsb,
                $ty;
                #[$field_doc]
            )*
        );
    };
    (
        @R,
        $(
            $field:ident,
            $msb:expr,
            $lsb:expr,
            $ty:ty;
            #[$field_doc:meta]
        )*
    ) => {
        impl Handle {
            $(
                #[$field_doc]
                pub fn $field(&self) -> $ty {
                    <$ty>::from_inner_bits(self.0, $msb, $lsb)
                }

            )*
        }
    };
    (
        @W,
        $(
            $field:ident,
            $msb:expr,
            $lsb:expr,
            $ty:ty;
            #[$field_doc:meta]
        )*
    ) => {
        paste! {
            impl Handle {
                $(
                    #[$field_doc]
                    pub fn [<set_ $field>](&mut self, value: $ty) -> &mut Self {
                        self.0 = <$ty>::set_inner_bits(self.0, $msb, $lsb, value);
                        self
                    }

                )*
            }
        }

    };
}

macro_rules! impl_rw {
    (RO, $name:ident) => {
        impl_rw!(@R, $name);
    };
    (RW, $name:ident) => {
        impl_rw!(@R, $name);
        impl_rw!(@W, $name);
    };

    (@R, $name:ident) => {
        paste! {
            impl Readable for $name {
                type Handle = [<$name:lower>]::Handle;

                fn fill(data: u8) -> Self::Handle {
                    [<$name:lower>]::Handle(data)
                }
            }
        }
    };
    (@W, $name:ident) => {

        paste! {
            impl Default for [<$name:lower>]::Handle {
                fn default() -> Self {
                    [<$name:lower>]::Handle(0)
                }
            }
        }


        impl Writable for $name {
            fn into_raw(w: Self::Handle) -> u8 {
                w.0
            }
        }
    };
}

sys_register! {
    0x00, RO, DEVID { ///Device ID
        value, 7, 0, u8; ///Device ID
    }
    // 0X01 to 0X1C     Reserved; do not access
    0x1D, RW, THRESH_TAP { ///Tap threshold
        value, 7, 0, u8; ///Tap threshold
    }
    0x1E, RW, OFSX { ///X-axis offset
        value, 7, 0, i8; ///X-axis offset
    }
    0x1F, RW, OFSY { /// Y-axis offset
        value, 7, 0, i8; ///Y-axis offset
    }
    0x20, RW, OFSZ { ///Z-axis offset
        value, 7, 0, i8; ///Z-axis offset
    }
    0x21, RW, DUR { ///Tap duration
        value, 7, 0, u8; ///Tap duration
    }
    0x22, RW, Latent { ///Tap latency
        value, 7, 0, u8; ///Tap latency
    }
    0x23, RW, Window { ///Tap window
        value, 7, 0, u8; ///Tap window
    }
    0x24, RW, THRESH_ACT { ///Activity threshold
        value, 7, 0, u8; ///Activity threshold
    }
    0x25, RW, THRESH_INACT { ///Inactivity threshold
        value, 7, 0, u8; ///Inactivity threshold
    }
    0x26, RW, TIME_INACT { ///Inactivity time
        value, 7, 0, u8; ///Inactivity time
    }
    0x27, RW, ACT_INACT_CTL { ///Axis enable control for activity and inactivity detection
        act_ac_dc, 7, 7, bool; ///ACT ac/dc
        act_x_enable, 6, 6, bool; ///ACT_X enable
        act_y_enable, 5, 5, bool; ///ACT_Y enable
        act_z_enable, 4, 4, bool; ///ACT_Z enable
        inact_ac_dc, 3, 3, bool; ///INACT ac/dc
        inact_x_enable, 2, 2, bool; ///INACT_X enable
        inact_y_enable, 1, 1, bool; ///INACT_Y enable
        inact_z_enable, 0, 0, bool; ///INACT_Z enable
    }
    0x28, RW, THRESH_FF { ///Free-fall threshold
        value, 7, 0, u8; ///Free-fall threshold
    }
    0x29, RW, TIME_FF { ///Free-fall time
        value, 7, 0, u8; ///Free-fall time
    }
    0x2A, RW, TAP_AXES { ///Axis control for single tap/double tap
        // 0
        // 0
        // 0
        // 0
        suppress, 3, 3, bool; ///Suppress
        tap_x_enable, 2, 2, bool; ///TAP_X enable
        tap_y_enable, 1, 1, bool; ///TAP_Y enable
        tap_z_enable, 0, 0, bool; ///TAP_Z enable
    }
    0x2B, RO, ACT_TAP_STATUS { ///Source of single tap/double tap
        // 0
        act_x_source, 6, 6, bool; ///ACT_X source
        act_y_source, 5, 5, bool; ///ACT_Y source
        act_z_source, 4, 4, bool; ///ACT_Z source
        asleep, 3, 3, bool; ///Asleep
        tap_x_source, 2, 2, bool; ///TAP_X source
        tap_y_source, 1, 1, bool; ///TAP_Y source
        tap_z_source, 0, 0, bool; ///TAP_Z source

    }
    0x2C, RW, BW_RATE { ///Data rate and power mode control
        // 0
        // 0
        // 0
        low_power, 4, 4, bool; ///LOW_POWER,
        rate, 3, 0, OutputDataRateHz; ///Rate
    }
    0x2D, RW, POWER_CTL { ///Power-saving features control
        // 0
        // 0
        link, 5, 5, bool; ///Link
        auto_sleep, 4, 4, bool; ///AUTO_SLEEP
        measure, 3, 3, bool; ///Measure
        sleep, 2, 2, bool; ///Sleep
        wakeup, 1, 0, ReadingFrequencyHz; ///Wakeup
    }
    0x2E, RW, INT_ENABLE { ///Interrupt enable control
        data_ready, 7, 7, bool; ///DATA_READY
        single_tap, 6, 6, bool; ///SINGLE_TAP
        double_tap, 5, 5, bool; ///DOUBLE_TAP
        activity, 4, 4, bool; ///Activity
        inactivity, 3, 3, bool; ///Inactivity
        free_fall, 2, 2, bool; ///FREE_FALL
        watermark, 1, 1, bool; ///Watermark
        overrun, 0, 0, bool; ///Overrun
    }
    0x2F, RW, INT_MAP { ///Interrupt mapping control
        data_ready, 7, 7, bool; ///DATA_READY
        single_tap, 6, 6, bool; ///SINGLE_TAP
        double_tap, 5, 5, bool; ///DOUBLE_TAP
        activity, 4, 4, bool; ///Activity
        inactivity, 3, 3, bool; ///Inactivity
        free_fall, 2, 2, bool; ///FREE_FALL
        watermark, 1, 1, bool; ///Watermark
        overrun, 0, 0, bool; ///Overrun
    }
    0x30, RO, INT_SOURCE { ///Source of interrupts
        data_ready, 7, 7, bool; ///DATA_READY
        single_tap, 6, 6, bool; ///SINGLE_TAP
        double_tap, 5, 5, bool; ///DOUBLE_TAP
        activity, 4, 4, bool; ///Activity
        inactivity, 3, 3, bool; ///Inactivity
        free_fall, 2, 2, bool; ///FREE_FALL
        watermark, 1, 1, bool; ///Watermark
        overrun, 0, 0, bool; ///Overrun
    }
    0x31, RW, DATA_FORMAT { ///Data format control
        self_test, 7, 7, bool; ///SELF_TEST
        spi, 6, 6, bool; ///SPI
        int_invert, 5, 5, bool; ///INT_INVERT
        // 0
        full_res, 3, 3, bool; ///FULL_RES
        justify, 2, 2, bool; ///Justify
        range, 1, 0, GRange; ///Range
    }
    0x32, RO, DATAX0 { ///X-Axis Data 0
        value, 7, 0, u8; ///X-Axis Data 0
    }
    0x33, RO, DATAX1 { ///X-Axis Data 1
        value, 7, 0, u8; ///X-Axis Data 1
    }
    0x34, RO, DATAY0 { ///Y-Axis Data 0
        value, 7, 0, u8; ///Y-Axis Data 0
    }
    0x35, RO, DATAY1 { ///Y-Axis Data 1
        value, 7, 0, u8; ///Y-Axis Data 1
    }
    0x36, RO, DATAZ0 { ///Z-Axis Data 0
        value, 7, 0, u8; ///Z-Axis Data 0
    }
    0x37, RO, DATAZ1 { ///Z-Axis Data 1
        value, 7, 0, u8; ///Z-Axis Data 1
    }
    0x38, RW, FIFO_CTL { ///FIFO control
        fifo_mode, 7, 6, FIFOMode; ///FIFO_MODE
        trigger, 5, 5, bool; ///Trigger
        samples, 4, 0, u8; ///Samples
    }
    0x39, RO, FIFO_STATUS { ///FIFO status
        fifo_trig, 7, 7, bool; ///FIFO_TRIG
        // 0
        entries, 5, 0, u8; ///Entries
    }
}
