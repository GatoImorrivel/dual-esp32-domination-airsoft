use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};

use super::wifi::WifiConfig;

const NVS_NS: &str = "domination";
const NVS_KEY: &str = "wifi_cfg";
const MAX_BLOB_LEN: usize = 512;

pub fn load_wifi_config(nvs: &EspDefaultNvsPartition) -> Option<WifiConfig> {
    let mut storage = EspNvs::new(nvs.clone(), NVS_NS, true).ok()?;
    let mut buf = [0u8; MAX_BLOB_LEN];
    let len = storage.get_blob(NVS_KEY, &mut buf).ok()??.len();
    postcard::from_bytes::<WifiConfig>(&buf[..len]).ok()
}

pub fn save_wifi_config(nvs: &EspDefaultNvsPartition, config: &WifiConfig) -> anyhow::Result<()> {
    let mut storage = EspNvs::new(nvs.clone(), NVS_NS, true)?;
    let bytes = postcard::to_allocvec(config)?;
    storage.set_blob(NVS_KEY, &bytes)?;
    log::info!("Wi-Fi config saved to NVS");
    Ok(())
}
