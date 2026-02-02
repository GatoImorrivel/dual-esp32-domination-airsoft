use std::time::Duration;

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
use serde::{Deserialize, Serialize};

use crate::{
    app::{App, AppClient},
    server::{load_web, HttpServer, Json, Response},
    wifi::Wifi,
};

mod app;
mod game;
mod server;
mod wifi;

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
    esp_idf_svc::hal::task::block_on(async {});

    let mut server = HttpServer::new();
    load_web(&mut server);
    routes(&mut server);
    core::mem::forget(server);

    let mut mdns = EspMdns::take().unwrap();
    mdns.set_hostname("sandi-dominacao").unwrap();
    mdns.add_service(Some("Sandi Dominacao"), "_http", "_tcp", 80, &[])
        .unwrap();
    core::mem::forget(mdns);

    std::thread::Builder::new()
        .stack_size(16 * 1024)
        .spawn(move || {
            let app = App::new(wifi);
            app.run(|_app| Ok(()));
        })
        .unwrap();

    loop {
        heap_report();
        std::thread::sleep(Duration::from_secs(10));
    }
}

fn routes(server: &mut HttpServer) {
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    struct EmptyRequest {}

    server.get("/game/progress", || {
        let client = AppClient::get();
        let progress = client.get_match_progress()?;
        Ok(Json::new(&progress)?.into())
    });

    server.post("/game/start", |_: EmptyRequest| {
        let client = AppClient::get();
        client.start_game()?;
        Ok(Response::ok())
    });

    server.post("/game/stop", |_: EmptyRequest| {
        let client = AppClient::get();
        client.stop_game()?;
        Ok(Response::ok())
    });
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
