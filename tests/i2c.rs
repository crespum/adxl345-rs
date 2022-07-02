use adxl345_hal::data_bus::i2c as adxl_i2c;
use adxl345_hal::register as reg;
use adxl345_hal::register::Register;
use adxl345_hal::ADXL345;

use embedded_hal_mock::i2c;

#[test]
fn read_devid() {
    const ADDRESS: u8 = adxl_i2c::Address::Default as u8;
    const DEVICE_ID: u8 = 0b1110_0101;
    let expect = vec![i2c::Transaction::write_read(
        ADDRESS,
        vec![reg::DEVID::ADDRESS],
        vec![DEVICE_ID],
    )];

    let mock = i2c::Mock::new(&expect);

    let mut device = ADXL345::from_i2c(mock, adxl_i2c::Address::Default);

    assert_eq!(device.devid().read().unwrap().value(), DEVICE_ID);

    device.destroy().done();
}
