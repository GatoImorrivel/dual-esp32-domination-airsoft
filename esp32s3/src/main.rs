use std::time::Duration;

use crate::{
    app::App,
    hardware::{input::InputButton, wifi::Wifi},
    http::{
        routes::routes,
        server::{load_web, HttpServer},
    },
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::Peripherals,
    mdns::EspMdns,
    nvs::EspDefaultNvsPartition,
    sys::{
        heap_caps_get_free_size, heap_caps_get_largest_free_block, MALLOC_CAP_INTERNAL,
        MALLOC_CAP_SPIRAM,
    },
    timer::EspTaskTimerService,
    wifi::{AsyncWifi, EspWifi},
};

mod app;
mod game;
mod hardware;
mod http;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
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

    let mut mdns = EspMdns::take().unwrap();
    mdns.set_hostname("sandi-dominacao").unwrap();
    mdns.add_service(Some("Sandi Dominacao"), "_http", "_tcp", 80, &[])
        .unwrap();
    core::mem::forget(mdns);

    let red_btn = InputButton::new(peripherals.pins.gpio8, 50)?;
    let blue_btn = InputButton::new(peripherals.pins.gpio18, 50)?;

    std::thread::Builder::new()
        .stack_size(16 * 1024)
        .spawn(move || {
            esp_idf_svc::hal::task::block_on(async {
                wifi.ap_mode().await.unwrap();
            });
            let app = App::new(wifi);
            app.run(move |app| {
                if red_btn.is_pressed() {
                    log::info!("Red team pressed");
                    app.mut_game().button_press(game::Team::Red);
                }

                if blue_btn.is_pressed() {
                    log::info!("Blue team pressed");
                    app.mut_game().button_press(game::Team::Blue);
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
