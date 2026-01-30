use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{gpio::{AnyIOPin, Gpio0, Gpio1, Gpio17}, prelude::Peripherals, uart::{UART2, Uart, UartDriver}},
    nvs::EspDefaultNvsPartition,
};

use crate::bt::BluetoothAudio;

mod bt;
mod uart;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let (_wifi_modem, bt_modem) = peripherals.modem.split();

    let bt = BluetoothAudio::init(bt_modem, Some(nvs.clone()))?;
    let uart = UartDriver::new(
        peripherals.uart2,
        peripherals.pins.gpio17,
        peripherals.pins.gpio16,
        Option::<Gpio0>::None,
        Option::<Gpio1>::None,
        &esp_idf_svc::hal::uart::config::Config {
            mode: esp_idf_svc::hal::uart::config::Mode::UART,
            ..Default::default()
        },
    );

    Ok(())
}
