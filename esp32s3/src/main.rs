use esp_idf_svc::{
    eventloop::EspSystemEventLoop, hal::{delay::FreeRtos, prelude::Peripherals}, mdns::EspMdns, nvs::EspDefaultNvsPartition, sys::{MALLOC_CAP_INTERNAL, MALLOC_CAP_SPIRAM, heap_caps_get_free_size, heap_caps_get_largest_free_block}, timer::EspTaskTimerService, wifi::{AsyncWifi, EspWifi}
};

use crate::wifi::Wifi;

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
    esp_idf_svc::hal::task::block_on(async {
    });



    let mut mdns = EspMdns::take()?;
    mdns.set_hostname("dominacao")?;
    mdns.add_service(Some("Sandi Dominacao"), "_http", "_tcp", 80, &[])?;

    loop {
        heap_report();
        FreeRtos::delay_ms(1000);
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
