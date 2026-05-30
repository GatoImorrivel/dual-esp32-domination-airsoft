//! Classic BT A2DP source. PCM: S16LE stereo 44.1 kHz.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use anyhow::Result;
use domination_uart::BtDevice;
use esp_idf_svc::hal::cpu::Core;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::task::thread::ThreadSpawnConfiguration;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use esp_idf_svc::{
    bt::{
        a2dp::{A2dpEvent, ConnectionStatus, EspA2dp, Source},
        gap::{EspGap, InqMode},
        BdAddr, BtClassic, BtDriver,
    },
    hal::{modem::BluetoothModemPeripheral, peripheral::Peripheral},
    sys::{
        esp_a2d_media_ctrl, esp_a2d_media_ctrl_t_ESP_A2D_MEDIA_CTRL_START,
        esp_a2d_media_ctrl_t_ESP_A2D_MEDIA_CTRL_STOP, vRingbufferReturnItem, xRingbufferCreate,
        xRingbufferReceiveUpTo, xRingbufferSend, RingbufHandle_t,
        RingbufferType_t_RINGBUF_TYPE_BYTEBUF,
    },
};

type BtClassicDriver = BtDriver<'static, BtClassic>;
type EspBtClassicGap = EspGap<'static, BtClassic, Arc<BtClassicDriver>>;

const NVS_NS: &str = "bt";
const NVS_KEY_ADDR: &str = "paired_addr";
const NVS_KEY_NAME: &str = "paired_name";
/// A2DP byte ringbuffer (heap); 64 KiB leaves room for one 12 KiB UART pthread after BT init.
const RINGBUF_BYTES: usize = 64 * 1024;

static ACTIVE_PLAY_ID: AtomicU32 = AtomicU32::new(0);

pub(crate) fn configure_app_pthread(name: &'static [u8], stack_size: usize) {
    let mut conf = ThreadSpawnConfiguration::default();
    conf.name = Some(name);
    conf.stack_size = stack_size;
    conf.priority = 5;
    conf.inherit = false;
    conf.pin_to_core = Some(Core::Core0);
    conf
        .set()
        .unwrap_or_else(|e| panic!("pthread config for {}: {e:?}", core::str::from_utf8(name).unwrap_or("?")));
}

pub(crate) fn active_play_id() -> u32 {
    ACTIVE_PLAY_ID.load(Ordering::Relaxed)
}

#[derive(Copy, Clone)]
struct Ringbuf(RingbufHandle_t);

unsafe impl Send for Ringbuf {}
unsafe impl Sync for Ringbuf {}

pub struct BluetoothAudio {
    gap: EspBtClassicGap,
    a2dp: EspA2dp<'static, BtClassic, Arc<BtClassicDriver>, Source>,
    ring_buf: Arc<Ringbuf>,
    connected: AtomicBool,
    connected_addr: RwLock<Option<BdAddr>>,
    paired: RwLock<Option<BtDevice>>,
    nvs: Mutex<EspNvs<NvsDefault>>,
}

impl BluetoothAudio {
    pub fn init<B: BluetoothModemPeripheral>(
        modem: impl Peripheral<P = B> + 'static,
        nvs: Option<EspDefaultNvsPartition>,
    ) -> anyhow::Result<Arc<Self>> {
        let nvs_partition = nvs.ok_or_else(|| anyhow::anyhow!("NVS required for BT"))?;
        let app_nvs = EspNvs::new(nvs_partition.clone(), NVS_NS, true)?;

        let bt = Arc::new(BluetoothAudio::new(modem, Some(nvs_partition), app_nvs)?);
        log::info!("Init Bluetooth Audio");

        let paired = bt.load_paired_from_nvs();
        if let Some(ref device) = paired {
            *bt.paired.write().unwrap() = Some(device.clone());
            log::info!("Restored paired device {:?}, scheduling A2DP connect", device);
            let bt_connect = bt.clone();
            let addr = BdAddr::from_bytes(device.addr);
            std::thread::Builder::new()
                .name("bt_reconnect".into())
                .stack_size(8 * 1024)
                .spawn(move || {
                    configure_app_pthread(b"bt_reconn\0", 8 * 1024);
                    for attempt in 1..=3 {
                        match bt_connect.a2dp_connect(&addr) {
                            Ok(()) => {
                                log::info!("A2DP connect initiated (attempt {attempt})");
                                break;
                            }
                            Err(e) => {
                                log::warn!("A2DP connect attempt {attempt} failed: {e:#}");
                                FreeRtos::delay_ms(2000);
                            }
                        }
                    }
                })
                .ok();
        }

        let a2dp_bt = bt.clone();
        bt.a2dp
            .subscribe(move |ev| Self::a2dp_event_handler(a2dp_bt.clone(), ev))?;

        Ok(bt)
    }

    fn new<B: BluetoothModemPeripheral>(
        modem: impl Peripheral<P = B> + 'static,
        nvs: Option<EspDefaultNvsPartition>,
        app_nvs: EspNvs<NvsDefault>,
    ) -> Result<Self> {
        let driver = Arc::new(BtDriver::new(modem, nvs)?);
        driver.set_device_name("Esp32dominacao")?;
        let gap = EspGap::new(driver.clone())?;
        gap.request_variable_pin()?;
        let handle = unsafe {
            xRingbufferCreate(RINGBUF_BYTES, RingbufferType_t_RINGBUF_TYPE_BYTEBUF)
        };
        if handle.is_null() {
            anyhow::bail!("xRingbufferCreate({RINGBUF_BYTES}) failed");
        }
        let a2dp = EspA2dp::new_source(driver)?;

        Ok(Self {
            gap,
            a2dp,
            ring_buf: Arc::new(Ringbuf(handle)),
            connected: AtomicBool::new(false),
            connected_addr: RwLock::new(None),
            paired: RwLock::new(None),
            nvs: Mutex::new(app_nvs),
        })
    }

    fn a2dp_event_handler(bt: Arc<Self>, ev: A2dpEvent) -> usize {
        match ev {
            A2dpEvent::ConnectionState {
                bd_addr,
                status,
                disconnect_abnormal,
            } => {
                if disconnect_abnormal {
                    log::warn!("A2DP abnormal disconnect from {bd_addr}");
                }
                match status {
                    ConnectionStatus::Connected => {
                        bt.connected.store(true, Ordering::SeqCst);
                        *bt.connected_addr.write().unwrap() = Some(bd_addr);
                        unsafe {
                            esp_a2d_media_ctrl(esp_a2d_media_ctrl_t_ESP_A2D_MEDIA_CTRL_START)
                        };
                        log::info!("A2DP connected to {bd_addr}");
                    }
                    ConnectionStatus::Disconnected => {
                        bt.connected.store(false, Ordering::SeqCst);
                        *bt.connected_addr.write().unwrap() = None;
                        log::info!("A2DP disconnected from {bd_addr}");
                    }
                    _ => {}
                }
                1
            }
            A2dpEvent::SourceData(buffer) => {
                let mut filled = 0usize;
                unsafe {
                    while filled < buffer.len() {
                        let mut size = 0;
                        let item = xRingbufferReceiveUpTo(
                            bt.ring_buf.0,
                            &mut size,
                            0,
                            buffer.len() - filled,
                        );
                        if item.is_null() {
                            break;
                        }
                        core::ptr::copy_nonoverlapping(
                            item as *const u8,
                            buffer.as_mut_ptr().add(filled),
                            size,
                        );
                        vRingbufferReturnItem(bt.ring_buf.0, item);
                        filled += size;
                    }
                    if filled < buffer.len() {
                        core::ptr::write_bytes(
                            buffer.as_mut_ptr().add(filled),
                            0,
                            buffer.len() - filled,
                        );
                    }
                }
                buffer.len()
            }
            any => {
                log::info!("{any:?}");
                1
            }
        }
    }

    /// Non-blocking send into the A2DP ringbuffer; returns false if full.
    pub fn try_send_bytes(&self, pcm: &[u8]) -> bool {
        unsafe {
            xRingbufferSend(
                self.ring_buf.0,
                pcm.as_ptr() as *const _,
                pcm.len(),
                0,
            ) != 0
        }
    }

    fn flush_ringbuffer(&self) {
        unsafe {
            let mut size = 0;
            loop {
                let item = xRingbufferReceiveUpTo(self.ring_buf.0, &mut size, 0, usize::MAX);
                if item.is_null() {
                    break;
                }
                vRingbufferReturnItem(self.ring_buf.0, item);
            }
        }
    }

    pub fn stop_playback(&self) {
        ACTIVE_PLAY_ID.store(0, Ordering::SeqCst);
        self.flush_ringbuffer();
    }

    pub fn arm_play_stream(&self, play_id: u32) {
        ACTIVE_PLAY_ID.store(play_id, Ordering::SeqCst);
        if self.connected.load(Ordering::SeqCst) {
            unsafe {
                esp_a2d_media_ctrl(esp_a2d_media_ctrl_t_ESP_A2D_MEDIA_CTRL_START);
            }
        }
    }

    pub fn a2dp_connect(&self, addr: &BdAddr) -> Result<()> {
        self.a2dp.connect_source(addr)?;
        Ok(())
    }

    fn wait_until_disconnected(&self, timeout_ms: u32) {
        let mut waited = 0u32;
        while self.connected.load(Ordering::SeqCst) && waited < timeout_ms {
            FreeRtos::delay_ms(50);
            waited += 50;
        }
        if self.connected.load(Ordering::SeqCst) {
            log::warn!(
                "A2DP disconnect wait timed out after {timeout_ms}ms; clearing local state"
            );
            self.connected.store(false, Ordering::SeqCst);
            *self.connected_addr.write().unwrap() = None;
        }
    }

    fn wait_until_connected(&self, target: &BdAddr, timeout_ms: u32) -> Result<()> {
        let target_bytes = target.addr();
        let mut waited = 0u32;
        while waited < timeout_ms {
            if self.connected.load(Ordering::SeqCst)
                && self
                    .connected_addr
                    .read()
                    .unwrap()
                    .as_ref()
                    .is_some_and(|a| a.addr() == target_bytes)
            {
                return Ok(());
            }
            FreeRtos::delay_ms(100);
            waited += 100;
        }
        anyhow::bail!("A2DP connect to {target} timed out after {timeout_ms}ms");
    }

    /// Disconnect any existing link, then connect and wait until the target is up.
    pub fn connect_to_device(&self, target: &BdAddr) -> Result<()> {
        let target_bytes = target.addr();
        let connected = self.connected.load(Ordering::SeqCst);
        let conn_addr = self.connected_addr.read().unwrap().clone();
        let paired_addr = self.paired.read().unwrap().as_ref().map(|d| d.addr);

        if connected && conn_addr.as_ref().is_some_and(|a| a.addr() == target_bytes) {
            return Ok(());
        }

        let needs_teardown = connected
            || conn_addr.is_some()
            || paired_addr.is_some_and(|a| a != target_bytes);

        if needs_teardown {
            self.a2dp_disconnect()?;
            self.wait_until_disconnected(8000);
            FreeRtos::delay_ms(1000);
        }

        self.a2dp_connect(target)?;
        self.wait_until_connected(target, 25_000)?;
        Ok(())
    }

    pub fn a2dp_disconnect(&self) -> Result<()> {
        self.stop_playback();

        // Bluedroid may call the source data callback with invalid buf during stop/flush.
        self.a2dp.clear_source_data_callback()?;

        if self.connected.load(Ordering::SeqCst) {
            unsafe {
                esp_a2d_media_ctrl(esp_a2d_media_ctrl_t_ESP_A2D_MEDIA_CTRL_STOP);
            }
            FreeRtos::delay_ms(300);
        }

        let addr = self
            .connected_addr
            .read()
            .unwrap()
            .or_else(|| {
                self.paired
                    .read()
                    .unwrap()
                    .as_ref()
                    .map(|d| BdAddr::from_bytes(d.addr))
            });
        if let Some(addr) = addr {
            self.a2dp.disconnect_source(&addr)?;
            FreeRtos::delay_ms(150);
        }
        self.connected.store(false, Ordering::SeqCst);
        *self.connected_addr.write().unwrap() = None;

        self.a2dp.restore_source_data_callback()?;
        Ok(())
    }

    pub fn connection_state(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    pub fn paired_device(&self) -> Option<BtDevice> {
        self.paired.read().unwrap().clone()
    }

    pub fn set_paired_device(&self, device: Option<BtDevice>) -> Result<()> {
        *self.paired.write().unwrap() = device.clone();
        let mut nvs = self.nvs.lock().unwrap();
        match device {
            Some(d) => {
                nvs.set_blob(NVS_KEY_ADDR, &d.addr)?;
                if let Some(name) = &d.name {
                    nvs.set_str(NVS_KEY_NAME, name)?;
                } else {
                    let _ = nvs.remove(NVS_KEY_NAME);
                }
            }
            None => {
                let _ = nvs.remove(NVS_KEY_ADDR);
                let _ = nvs.remove(NVS_KEY_NAME);
            }
        }
        Ok(())
    }

    fn load_paired_from_nvs(&self) -> Option<BtDevice> {
        let nvs = self.nvs.lock().ok()?;
        let mut addr = [0u8; 6];
        if nvs.get_blob(NVS_KEY_ADDR, &mut addr).ok()?.is_none() {
            return None;
        }
        let mut name_buf = [0u8; 64];
        let name = nvs
            .get_str(NVS_KEY_NAME, &mut name_buf)
            .ok()
            .flatten()
            .map(|s| s.to_string());
        Some(BtDevice { name, addr })
    }

    pub fn discover_devices(
        &self,
        duration: u8,
        max_responses: usize,
    ) -> anyhow::Result<Vec<BtDevice>> {
        let devices: Arc<Mutex<Vec<BtDevice>>> = Arc::new(Mutex::new(vec![]));
        let devices_handler = devices.clone();

        self.gap.subscribe(move |event| match event {
            esp_idf_svc::bt::gap::GapEvent::DeviceDiscovered { bd_addr, props } => {
                let mut devices = devices_handler.lock().unwrap();
                if devices.iter().any(|d| d.addr == bd_addr.addr()) {
                    return;
                }
                let mut device_name = None;
                for prop in props {
                    if let esp_idf_svc::bt::gap::DeviceProp::Eir(eir) = prop.prop() {
                        if let Some(name) = eir.local_name::<BtClassic, BtClassicDriver>() {
                            device_name = Some(name.to_owned());
                        }
                    }
                }
                devices.push(BtDevice {
                    name: device_name,
                    addr: bd_addr.addr(),
                });
            }
            _ => {}
        })?;

        if let Err(e) = self
            .gap
            .start_discovery(InqMode::General, duration, max_responses)
        {
            let _ = self.gap.unsubscribe();
            return Err(e.into());
        }
        FreeRtos::delay_ms(duration as u32 * 1000);
        let stop_result = self.gap.stop_discovery();
        self.gap.unsubscribe()?;
        stop_result?;

        let devices = Arc::try_unwrap(devices)
            .map_err(|_| anyhow::anyhow!("discovery callback still held"))?
            .into_inner()
            .unwrap();
        Ok(devices)
    }
}
