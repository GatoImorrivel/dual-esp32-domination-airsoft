use std::time::Duration;

use esp_idf_svc::{
    hal::{
        gpio::{Gpio0, Gpio1},
        prelude::Peripherals,
        uart::config::{Config as UartConfig, Mode},
        uart::UartDriver,
    },
    nvs::EspDefaultNvsPartition,
};

use domination_uart::BAUD_RATE;

use crate::bt::BluetoothAudio;

mod audio;
mod bt;
mod uart;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let (_wifi_modem, bt_modem) = peripherals.modem.split();

    let bt = BluetoothAudio::init(bt_modem, Some(nvs))?;

    let uart_config = UartConfig {
        baudrate: esp_idf_svc::hal::units::Hertz(BAUD_RATE),
        mode: Mode::UART,
        ..Default::default()
    };

    let uart = UartDriver::new(
        peripherals.uart2,
        peripherals.pins.gpio17,
        peripherals.pins.gpio16,
        Option::<Gpio0>::None,
        Option::<Gpio1>::None,
        &uart_config,
    )?;

    uart::spawn_bridge(bt, uart);

    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}
