// TODO: Cannot test until https://github.com/dbrgn/embedded-hal-mock/issues/25 gets fixed
// use adxl345_hal::data_bus::SPIBus;
// use adxl345_hal::register as reg;
// use adxl345_hal::register::Register;
// use adxl345_hal::ADXL345;

// use embedded_hal_mock::pin::{Mock as PinMock, State as PinState, Transaction as PinTransaction};
// use embedded_hal_mock::spi::{Mock as SpiMock, Transaction as SpiTransaction, self};

// #[test]
// fn read_devid() {
//     const DEVICE_ID: u8 = 0b1110_0101;
//     let pin_expect = vec![
//         PinTransaction::get(PinState::High),
//         PinTransaction::set(PinState::Low),
//         PinTransaction::set(PinState::High)
//     ];

//     let pin_mock = PinMock::new(&pin_expect);

//     let spi_expect = vec![
//         SpiTransaction::send(u16::from_be_bytes([reg::DEVID, 0])),
//         SpiTransaction::read(u16::from_be_bytes([0, DEVIDE_ID]))
//     ];

//     let mock = spi::Mock::new(&spi_expect);

//     let mut device = ADXL345::from_spi_cs(mock, pin_mock);

//     assert_eq!(device.devid().read().unwrap().value(), DEVICE_ID);

//     device.destroy().done();
// }
