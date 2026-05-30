//! Single-threaded UART owner for the BT coprocessor.
//!
//! Wiring: S3 UART1 TX=GPIO4, RX=GPIO5 ↔ ESP32 UART2 TX=GPIO17, RX=GPIO16, common GND.
//! Baud: [`domination_uart::BAUD_RATE`] (921600). PCM: S16LE stereo 44.1 kHz (ESP32 `bt.rs`).

use esp_idf_svc::hal::uart::UartDriver as _;
use esp_idf_svc::io::Write;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use domination_uart::{
    codec::{decode_frames, encode_frame},
    format_mac, parse_mac,
    protocol::{Opcode, Request, Response},
    BtDevice, MAX_RX_ACCUM,
};
use esp_idf_svc::hal::uart::UartDriver;
use postcard;

use super::{AudioSink, BtSinksResponse};

static DISPATCHER_TX: OnceLock<Sender<DispatcherCmd>> = OnceLock::new();
static NEXT_PLAY_ID: AtomicU32 = AtomicU32::new(1);
static CURRENT_PLAY_ID: AtomicU32 = AtomicU32::new(0);

struct Cache {
    paired: Option<AudioSink>,
    discovered: Vec<AudioSink>,
    scanning: bool,
    connected: bool,
}

static CACHE: OnceLock<RwLock<Cache>> = OnceLock::new();

fn cache() -> &'static RwLock<Cache> {
    CACHE.get_or_init(|| {
        RwLock::new(Cache {
            paired: None,
            discovered: vec![],
            scanning: false,
            connected: false,
        })
    })
}

enum DispatcherCmd {
    Scan,
    Connect {
        addr: [u8; 6],
        name: Option<String>,
        reply: mpsc::Sender<Result<()>>,
    },
    Disconnect {
        reply: mpsc::Sender<Result<()>>,
    },
    GetStatus {
        reply: mpsc::Sender<Result<(Option<AudioSink>, bool)>>,
    },
    PlaySound { play_id: u32, sound_id: u8 },
}

pub fn init(uart: UartDriver<'static>) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("bt_dispatch".into())
        .stack_size(12 * 1024)
        .spawn(move || dispatcher_loop(rx, uart))
        .map_err(|e| anyhow!("spawn bt dispatcher: {e}"))?;
    DISPATCHER_TX
        .set(tx)
        .map_err(|_| anyhow!("dispatcher already init"))?;
    let _ = cache();
    Ok(())
}

fn dispatcher_tx() -> Result<&'static Sender<DispatcherCmd>> {
    DISPATCHER_TX
        .get()
        .ok_or_else(|| anyhow!("BT dispatcher not initialized"))
}

pub fn start_scan() -> Result<()> {
    {
        let mut c = cache().write().map_err(|e| anyhow!("{e}"))?;
        if c.scanning {
            return Ok(());
        }
        c.scanning = true;
    }
    dispatcher_tx()?.send(DispatcherCmd::Scan)?;
    Ok(())
}

fn is_scanning() -> Result<bool> {
    Ok(cache().read().map_err(|e| anyhow!("{e}"))?.scanning)
}

/// Read coprocessor link state into the cache (skipped while a scan owns the UART).
pub fn refresh_status_if_idle() -> Result<()> {
    if is_scanning()? {
        return Ok(());
    }
    refresh_status()
}

pub fn list_sinks_cached() -> Result<BtSinksResponse> {
    let c = cache().read().map_err(|e| anyhow!("{e}"))?;
    Ok(BtSinksResponse {
        paired: c.paired.clone(),
        discovered: c.discovered.clone(),
        scanning: c.scanning,
        connected: c.connected,
    })
}

/// Cached sinks plus a live `GetStatus` from the coprocessor when not scanning.
pub fn list_sinks_live() -> Result<BtSinksResponse> {
    if let Err(e) = refresh_status_if_idle() {
        log::warn!("BT status refresh failed: {e:#}");
    }
    list_sinks_cached()
}

pub fn pair_sink_dispatch(address: &str) -> Result<BtSinksResponse> {
    {
        let c = cache().read().map_err(|e| anyhow!("{e}"))?;
        if c.scanning {
            return Err(anyhow!(
                "Bluetooth scan in progress; wait for scan to finish before pairing"
            ));
        }
    }

    let addr = parse_mac(address)?;
    let name = cache()
        .read()
        .map_err(|e| anyhow!("{e}"))?
        .discovered
        .iter()
        .find(|s| s.address.eq_ignore_ascii_case(address))
        .and_then(|s| s.name.clone());

    let (tx, rx) = mpsc::channel();
    dispatcher_tx()?.send(DispatcherCmd::Connect {
        addr,
        name,
        reply: tx,
    })?;
    rx.recv_timeout(Duration::from_secs(60))
        .map_err(|_| anyhow!("connect timeout"))??;

    list_sinks_cached()
}

pub fn unpair_sink_dispatch() -> Result<BtSinksResponse> {
    let (tx, rx) = mpsc::channel();
    dispatcher_tx()?.send(DispatcherCmd::Disconnect { reply: tx })?;
    rx.recv_timeout(Duration::from_secs(20))
        .map_err(|_| anyhow!("disconnect timeout"))??;

    list_sinks_cached()
}

pub fn request_play_sound(sound_id: u8) -> Result<u32> {
    let play_id = NEXT_PLAY_ID.fetch_add(1, Ordering::SeqCst);
    CURRENT_PLAY_ID.store(play_id, Ordering::SeqCst);
    dispatcher_tx()?.send(DispatcherCmd::PlaySound { play_id, sound_id })?;
    Ok(play_id)
}

pub(crate) fn refresh_status() -> Result<()> {
    let (tx, rx) = mpsc::channel();
    dispatcher_tx()?.send(DispatcherCmd::GetStatus { reply: tx })?;
    let (paired, connected) = rx.recv_timeout(Duration::from_secs(5))??;
    let mut c = cache().write().map_err(|e| anyhow!("{e}"))?;
    c.paired = paired;
    c.connected = connected;
    Ok(())
}

fn device_to_sink(d: BtDevice) -> AudioSink {
    AudioSink {
        address: format_mac(d.addr),
        name: d.name,
    }
}

fn append_rx(acc: &mut Vec<u8>, data: &[u8]) {
    acc.extend_from_slice(data);
    if acc.len() > MAX_RX_ACCUM {
        let drop = acc.len() - MAX_RX_ACCUM;
        acc.drain(..drop);
        log::warn!("uart rx acc truncated {drop} bytes");
    }
}

fn dispatcher_loop(rx: Receiver<DispatcherCmd>, mut uart: UartDriver<'static>) {
    let mut seq: u8 = 0;
    let mut acc = Vec::new();
    let mut read_buf = [0u8; 1024];

    while let Ok(cmd) = rx.recv() {
        match cmd {
            DispatcherCmd::Scan => {
                match do_scan(&mut uart, &mut seq, &mut acc, &mut read_buf) {
                    Ok(devices) => {
                        let sinks: Vec<AudioSink> =
                            devices.into_iter().map(device_to_sink).collect();
                        if let Ok(mut c) = cache().write() {
                            c.discovered = sinks;
                            c.scanning = false;
                        }
                        log::info!("BT scan done: {} device(s)", {
                            cache().read().map(|c| c.discovered.len()).unwrap_or(0)
                        });
                        if let Err(e) =
                            refresh_status_from_loop(&mut uart, &mut seq, &mut acc, &mut read_buf)
                        {
                            log::warn!("post-scan status refresh failed: {e:#}");
                        }
                    }
                    Err(e) => {
                        log::error!("BT scan failed: {e:#}");
                        acc.clear();
                        if let Ok(mut c) = cache().write() {
                            c.scanning = false;
                        }
                    }
                }
            }
            DispatcherCmd::Connect { addr, name, reply } => {
                let result =
                    do_connect(&mut uart, &mut seq, &mut acc, &mut read_buf, addr, name.clone());
                let ok = result.is_ok();
                if reply.send(result).is_err() {
                    log::warn!("connect reply dropped");
                }
                if ok {
                    if let Err(e) = refresh_status_from_loop(&mut uart, &mut seq, &mut acc, &mut read_buf) {
                        log::warn!("post-connect status refresh failed: {e:#}");
                    }
                }
            }
            DispatcherCmd::Disconnect { reply } => {
                let result = do_disconnect(&mut uart, &mut seq, &mut acc, &mut read_buf);
                if reply.send(result).is_err() {
                    log::warn!("disconnect reply dropped");
                }
                if let Err(e) = refresh_status_from_loop(&mut uart, &mut seq, &mut acc, &mut read_buf) {
                    log::warn!("post-disconnect status refresh failed: {e:#}");
                }
            }
            DispatcherCmd::GetStatus { reply } => {
                let result = do_get_status(&mut uart, &mut seq, &mut acc, &mut read_buf);
                if let Ok((ref paired, connected)) = result {
                    if let Ok(mut c) = cache().write() {
                        c.paired = paired.clone();
                        c.connected = connected;
                    }
                }
                if reply.send(result).is_err() {
                    log::warn!("get status reply dropped");
                }
            }
            DispatcherCmd::PlaySound { play_id, sound_id } => {
                let mut latest_id = play_id;
                let mut latest_sound = sound_id;
                while let Ok(DispatcherCmd::PlaySound { play_id, sound_id }) = rx.try_recv() {
                    latest_id = play_id;
                    latest_sound = sound_id;
                    CURRENT_PLAY_ID.store(play_id, Ordering::SeqCst);
                }
                if let Err(e) = do_play_sound(
                    &mut uart,
                    &mut seq,
                    &mut acc,
                    &mut read_buf,
                    latest_id,
                    latest_sound,
                ) {
                    log::error!("play sound {latest_sound} id={latest_id} failed: {e:#}");
                    if let Err(refresh_err) =
                        refresh_status_from_loop(&mut uart, &mut seq, &mut acc, &mut read_buf)
                    {
                        log::warn!("post-play status refresh failed: {refresh_err:#}");
                    }
                }
            }
        }
    }
}

fn refresh_status_from_loop(
    uart: &mut UartDriver<'static>,
    seq: &mut u8,
    acc: &mut Vec<u8>,
    read_buf: &mut [u8],
) -> Result<()> {
    let (paired, connected) = do_get_status(uart, seq, acc, read_buf)?;
    let mut c = cache().write().map_err(|e| anyhow!("{e}"))?;
    c.paired = paired;
    c.connected = connected;
    Ok(())
}

fn next_seq(seq: &mut u8) -> u8 {
    let s = *seq;
    *seq = seq.wrapping_add(1);
    s
}

fn write_request(
    uart: &mut UartDriver<'static>,
    seq: u8,
    opcode: Opcode,
    req: &Request,
) -> Result<()> {
    let payload = postcard::to_allocvec(req)?;
    let frame = encode_frame(opcode, seq, false, &payload);
    uart.write_all(&frame)?;
    Ok(())
}

fn read_response(
    uart: &mut UartDriver<'static>,
    acc: &mut Vec<u8>,
    read_buf: &mut [u8],
    opcode: Opcode,
    expect_seq: u8,
    timeout: Duration,
) -> Result<Response> {
    let deadline = Instant::now() + timeout;
    loop {
        if Instant::now() > deadline {
            acc.clear();
            return Err(anyhow!("uart response timeout (opcode={opcode:?}, seq={expect_seq})"));
        }
        let n = UartDriver::read(uart, read_buf, 10).unwrap_or(0);
        if n > 0 {
            append_rx(acc, &read_buf[..n]);
        }
        if let Ok((frames, consumed)) = decode_frames(acc) {
            if consumed > 0 {
                acc.drain(..consumed);
            }
            for frame in frames {
                if !frame.is_response || frame.opcode != opcode || frame.seq != expect_seq {
                    continue;
                }
                return postcard::from_bytes(&frame.payload)
                    .map_err(|e| anyhow!("decode response: {e}"));
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn do_scan(
    uart: &mut UartDriver<'static>,
    seq: &mut u8,
    acc: &mut Vec<u8>,
    read_buf: &mut [u8],
) -> Result<Vec<BtDevice>> {
    let s = next_seq(seq);
    write_request(
        uart,
        s,
        Opcode::Scan,
        &Request::Scan {
            duration_secs: 10,
        },
    )?;
    let timeout = Duration::from_secs(15);
    match read_response(uart, acc, read_buf, Opcode::Scan, s, timeout)? {
        Response::ScanResult { devices } => Ok(devices),
        Response::Error { code } => Err(anyhow!("scan error: {:?}", code)),
        other => {
            acc.clear();
            Err(anyhow!("unexpected scan response: {:?}", other))
        }
    }
}

fn do_connect(
    uart: &mut UartDriver<'static>,
    seq: &mut u8,
    acc: &mut Vec<u8>,
    read_buf: &mut [u8],
    addr: [u8; 6],
    name: Option<String>,
) -> Result<()> {
    let s = next_seq(seq);
    write_request(
        uart,
        s,
        Opcode::Connect,
        &Request::Connect { addr, name },
    )?;
    let timeout = Duration::from_secs(60);
    match read_response(uart, acc, read_buf, Opcode::Connect, s, timeout)? {
        Response::Ok => Ok(()),
        Response::Error { code } => Err(anyhow!("connect: {:?}", code)),
        other => {
            log::warn!("unexpected connect response (seq={s}): {:?}", other);
            acc.clear();
            Err(anyhow!("unexpected connect response: {:?}", other))
        }
    }
}

fn do_disconnect(
    uart: &mut UartDriver<'static>,
    seq: &mut u8,
    acc: &mut Vec<u8>,
    read_buf: &mut [u8],
) -> Result<()> {
    let s = next_seq(seq);
    write_request(uart, s, Opcode::Disconnect, &Request::Disconnect)?;
    match read_response(
        uart,
        acc,
        read_buf,
        Opcode::Disconnect,
        s,
        Duration::from_secs(15),
    )? {
        Response::Ok => Ok(()),
        Response::Error { code } => Err(anyhow!("disconnect: {:?}", code)),
        other => {
            acc.clear();
            Err(anyhow!("unexpected disconnect response: {:?}", other))
        }
    }
}

fn do_get_status(
    uart: &mut UartDriver<'static>,
    seq: &mut u8,
    acc: &mut Vec<u8>,
    read_buf: &mut [u8],
) -> Result<(Option<AudioSink>, bool)> {
    let s = next_seq(seq);
    write_request(uart, s, Opcode::GetStatus, &Request::GetStatus)?;
    match read_response(
        uart,
        acc,
        read_buf,
        Opcode::GetStatus,
        s,
        Duration::from_secs(15),
    )? {
        Response::Status { paired, connected } => Ok((paired.map(device_to_sink), connected)),
        Response::Error { code } => Err(anyhow!("status error: {:?}", code)),
        other => {
            acc.clear();
            Err(anyhow!("unexpected status response: {:?}", other))
        }
    }
}

fn do_play_sound(
    uart: &mut UartDriver<'static>,
    seq: &mut u8,
    acc: &mut Vec<u8>,
    read_buf: &mut [u8],
    play_id: u32,
    sound_id: u8,
) -> Result<()> {
    if CURRENT_PLAY_ID.load(Ordering::SeqCst) != play_id {
        return Ok(());
    }

    let s = next_seq(seq);
    write_request(
        uart,
        s,
        Opcode::PlaySound,
        &Request::PlaySound { play_id, sound_id },
    )?;
    match read_response(
        uart,
        acc,
        read_buf,
        Opcode::PlaySound,
        s,
        Duration::from_secs(15),
    )? {
        Response::Ok => Ok(()),
        Response::Error { code } => Err(anyhow!("play sound: {:?}", code)),
        other => {
            acc.clear();
            Err(anyhow!("unexpected play sound response: {:?}", other))
        }
    }
}

pub fn ping_coprocessor(uart: &mut UartDriver<'static>) -> bool {
    let mut seq = 0u8;
    let mut acc = Vec::new();
    let mut read_buf = [0u8; 256];
    write_request(uart, seq, Opcode::Ping, &Request::Ping).is_ok()
        && matches!(
            read_response(
                uart,
                &mut acc,
                &mut read_buf,
                Opcode::Ping,
                seq,
                Duration::from_secs(5),
            ),
            Ok(Response::Pong)
        )
}
