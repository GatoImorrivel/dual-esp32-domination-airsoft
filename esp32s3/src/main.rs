use std::time::Duration;

use crate::{
    app::App,
    hardware::{input::InputButton, wifi::Wifi},
    http::{
        routes::{load_web, routes},
        server::HttpServer,
    },
};
use domination_uart::BAUD_RATE;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        prelude::Peripherals,
        uart::config::{Config as UartConfig, Mode},
    },
    nvs::EspDefaultNvsPartition,
    sys::{
        heap_caps_get_free_size, heap_caps_get_largest_free_block, MALLOC_CAP_INTERNAL,
        MALLOC_CAP_SPIRAM,
    },
    timer::EspTaskTimerService,
    wifi::{AsyncWifi, EspWifi},
};

mod app;
mod audio;
mod bt;
mod game;
mod hardware;
mod http;
mod middleware;

#[cfg(any(
    esp_idf_comp_mdns_enabled,
    esp_idf_comp_espressif__mdns_enabled
))]
fn init_mdns() -> anyhow::Result<()> {
    use esp_idf_svc::mdns::EspMdns;

    let mut mdns = EspMdns::take().unwrap();
    mdns.set_hostname("sandi-dominacao").unwrap();
    mdns.add_service(Some("Sandi Dominacao"), "_http", "_tcp", 80, &[])
        .unwrap();
    core::mem::forget(mdns);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    #[cfg(not(test))]
    {
        let uart_config = UartConfig {
            baudrate: esp_idf_svc::hal::units::Hertz(BAUD_RATE),
            mode: Mode::UART,
            ..Default::default()
        };
        let bt_uart = esp_idf_svc::hal::uart::UartDriver::new(
            peripherals.uart1,
            peripherals.pins.gpio4,
            peripherals.pins.gpio5,
            Option::<esp_idf_svc::hal::gpio::Gpio0>::None,
            Option::<esp_idf_svc::hal::gpio::Gpio1>::None,
            &uart_config,
        )?;
        bt::init_dispatcher(bt_uart)?;
        log::info!("BT UART bridge ready (TX=GPIO4, RX=GPIO5, {} baud)", BAUD_RATE);
        match bt::check_coprocessor() {
            Ok(()) => log::info!("BT coprocessor reachable"),
            Err(e) => log::warn!(
                "BT coprocessor not reachable (flash esp32, check GPIO4/5↔17/16, GND): {e:#}"
            ),
        }
    }
    let sys_loop = EspSystemEventLoop::take()?;
    let (wifi_modem, _bt_modem) = peripherals.modem.split();

    let wifi_timer = EspTaskTimerService::new()?;
    let async_wifi = AsyncWifi::wrap(
        EspWifi::new(wifi_modem, sys_loop.clone(), Some(nvs.clone()))?,
        sys_loop.clone(),
        wifi_timer,
    )?;

    let mut wifi = Wifi::init(async_wifi);
    let mut server = HttpServer::new();
    load_web(&mut server);
    routes(&mut server);
    core::mem::forget(server);

    #[cfg(any(
        esp_idf_comp_mdns_enabled,
        esp_idf_comp_espressif__mdns_enabled
    ))]
    init_mdns()?;

    let red_btn = InputButton::new(peripherals.pins.gpio8, 50)?;
    let blue_btn = InputButton::new(peripherals.pins.gpio18, 50)?;

    std::thread::Builder::new()
        .stack_size(16 * 1024)
        .spawn(move || {
            esp_idf_svc::hal::task::block_on(async {
                // wifi.ap_mode().await.unwrap();
                wifi.client_mode("Ruptura-2G", "5himikug@_2G").await.unwrap();
            });
            let app = App::new(wifi);
            app.run(move |app| {
                if red_btn.is_pressed() && app.mut_game().button_press(game::Team::Red) {
                    audio::play_team(game::Team::Red);
                }

                if blue_btn.is_pressed() && app.mut_game().button_press(game::Team::Blue) {
                    audio::play_team(game::Team::Blue);
                }

                Ok(())
            });
        })
        .unwrap();

    loop {
        heap_report();
        std::thread::sleep(Duration::from_secs(10));
    }
}

pub fn heap_report() {
    unsafe {
        let internal_free = heap_caps_get_free_size(MALLOC_CAP_INTERNAL);
        let internal_largest = heap_caps_get_largest_free_block(MALLOC_CAP_INTERNAL);

        let psram_free = heap_caps_get_free_size(MALLOC_CAP_SPIRAM);
        let psram_largest = heap_caps_get_largest_free_block(MALLOC_CAP_SPIRAM);

        log::info!(
            "HEAP | internal: {} free / {} largest | psram: {} free / {} largest",
            internal_free,
            internal_largest,
            psram_free,
            psram_largest,
        );
    }
}
