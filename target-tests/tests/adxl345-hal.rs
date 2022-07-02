#![no_std]
#![no_main]

extern crate defmt_rtt; // defmt transport
extern crate panic_probe; // panic handler

use adxl345_hal::{data_bus::DataBus, ADXL345};

struct State<B: DataBus> {
    adxl345: ADXL345<B>,
}

#[defmt_test::tests]
mod tests {
    use adxl345_hal::data_bus::DataBus;
    use rp_pico::hal;
    use rp_pico::hal::gpio;
    use rp_pico::hal::pac;
    use rp_pico::hal::prelude::*;

    use embedded_time::rate::*;

    use nb::block;

    use super::State;

    #[init]
    fn setup() -> State<impl DataBus> {
        let mut pac = pac::Peripherals::take().unwrap();
        let mut core = pac::CorePeripherals::take().unwrap();
        core.DCB.enable_trace();

        let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

        let clocks = hal::clocks::init_clocks_and_plls(
            rp_pico::XOSC_CRYSTAL_FREQ,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let sio = hal::Sio::new(pac.SIO);

        let pins = rp_pico::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        let _spi_sclk = pins.gpio10.into_mode::<gpio::FunctionSpi>();
        let _spi_mosi = pins.gpio11.into_mode::<gpio::FunctionSpi>();
        let _spi_miso = pins.gpio12.into_mode::<gpio::FunctionSpi>();
        let spi_cs = pins.gpio13.into_push_pull_output();

        let spi = hal::Spi::<_, _, 16>::new(pac.SPI1);

        let spi = spi.init(
            &mut pac.RESETS,
            clocks.peripheral_clock.freq(),
            2_500_000u32.Hz(),
            &embedded_hal::spi::MODE_3,
        );

        let adxl345 = adxl345_hal::ADXL345::from_spi_cs(spi, spi_cs);

        State { adxl345 }
    }
}
