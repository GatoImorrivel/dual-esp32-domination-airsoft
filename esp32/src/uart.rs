//! UART2 bridge: GPIO17 TX, GPIO16 RX @ [`domination_uart::BAUD_RATE`].

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use domination_uart::{
    codec::{decode_frames, encode_frame, FrameDecodeError},
    protocol::{ErrorCode, Opcode, Request, Response},
    BtDevice, MAX_CHUNK, MAX_DEVICES,
};
use esp_idf_svc::bt::BdAddr;
use esp_idf_svc::hal::uart::UartDriver;
use esp_idf_svc::io::Write;
use postcard;

use crate::audio;
use crate::bt::{active_play_id, configure_app_pthread, BluetoothAudio};

struct PendingPlay {
    play_id: u32,
    sound_id: u8,
    offset: usize,
}

pub fn spawn_bridge(bt: Arc<BluetoothAudio>, mut uart: UartDriver<'static>) {
    configure_app_pthread(b"uart_bt\0", 12 * 1024);
    std::thread::Builder::new()
        .name("uart_bt".into())
        .stack_size(12 * 1024)
        .spawn(move || {
            if let Err(e) = bridge_loop(bt, &mut uart) {
                log::error!("UART bridge exited: {e:#}");
            }
        })
        .expect("spawn uart bridge");
}

fn bridge_loop(bt: Arc<BluetoothAudio>, uart: &mut UartDriver<'static>) -> Result<()> {
    let mut rx_buf = vec![0u8; 2048];
    let mut acc = Vec::new();
    let mut busy = false;
    let mut pending_play: Option<PendingPlay> = None;

    loop {
        let n = esp_idf_svc::hal::uart::UartDriver::read(uart, &mut rx_buf, 10).unwrap_or(0);
        if n > 0 {
            acc.extend_from_slice(&rx_buf[..n]);
        }

        loop {
            match decode_frames(&acc) {
                Ok((frames, consumed)) => {
                    if consumed > 0 {
                        acc.drain(..consumed);
                    }
                    if frames.is_empty() {
                        break;
                    }
                    for frame in frames {
                        if frame.is_response {
                            continue;
                        }
                        let resp = handle_frame(
                            &bt,
                            &mut busy,
                            &mut pending_play,
                            frame.opcode,
                            &frame.payload,
                        );
                        let payload = postcard::to_allocvec(&resp)?;
                        let out = encode_frame(frame.opcode, frame.seq, true, &payload);
                        uart.write_all(&out)?;
                    }
                }
                Err(FrameDecodeError::NeedMoreBytes) => break,
                Err(FrameDecodeError::CrcMismatch)
                | Err(FrameDecodeError::BadVersion)
                | Err(FrameDecodeError::UnknownOpcode)
                | Err(FrameDecodeError::LengthMismatch)
                | Err(FrameDecodeError::BadSync) => {
                    if !acc.is_empty() {
                        acc.drain(..1);
                    }
                }
            }
        }

        tick_pending_play(&bt, &mut pending_play);
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn tick_pending_play(bt: &BluetoothAudio, pending: &mut Option<PendingPlay>) {
    let Some(p) = pending.as_mut() else {
        return;
    };
    if active_play_id() != p.play_id {
        *pending = None;
        return;
    }
    let Some(pcm) = audio::pcm_for_sound(p.sound_id) else {
        log::warn!("unknown sound_id {}", p.sound_id);
        *pending = None;
        return;
    };
    if p.offset >= pcm.len() {
        *pending = None;
        return;
    }
    let end = (p.offset + MAX_CHUNK).min(pcm.len());
    if bt.try_send_bytes(&pcm[p.offset..end]) {
        p.offset = end;
    }
}

fn handle_frame(
    bt: &Arc<BluetoothAudio>,
    busy: &mut bool,
    pending_play: &mut Option<PendingPlay>,
    opcode: Opcode,
    payload: &[u8],
) -> Response {
    if *busy && !matches!(opcode, Opcode::Ping) {
        return Response::Error {
            code: ErrorCode::Busy,
        };
    }

    let req: Request = match postcard::from_bytes(payload) {
        Ok(r) => r,
        Err(_) => {
            return Response::Error {
                code: ErrorCode::InvalidPayload,
            };
        }
    };

    match (opcode, req) {
        (Opcode::Ping, Request::Ping) => Response::Pong,

        (Opcode::Scan, Request::Scan { duration_secs }) => {
            *busy = true;
            let result = bt
                .discover_devices(duration_secs, MAX_DEVICES)
                .map(|mut devices| {
                    devices.truncate(MAX_DEVICES);
                    Response::ScanResult { devices }
                })
                .unwrap_or(Response::Error {
                    code: ErrorCode::ScanFailed,
                });
            *busy = false;
            result
        }

        (Opcode::Connect, Request::Connect { addr, name }) => {
            *busy = true;
            let bd = BdAddr::from_bytes(addr);
            let resp = match bt.a2dp_connect(&bd) {
                Ok(()) => {
                    let device = BtDevice { name, addr };
                    let _ = bt.set_paired_device(Some(device));
                    Response::Ok
                }
                Err(_) => Response::Error {
                    code: ErrorCode::ConnectFailed,
                },
            };
            *busy = false;
            resp
        }

        (Opcode::Disconnect, Request::Disconnect) => {
            let _ = bt.a2dp_disconnect();
            let _ = bt.set_paired_device(None);
            *pending_play = None;
            bt.stop_playback();
            Response::Ok
        }

        (Opcode::GetStatus, Request::GetStatus) => Response::Status {
            paired: bt.paired_device(),
            connected: bt.connection_state(),
        },

        (Opcode::PlaySound, Request::PlaySound { play_id, sound_id }) => {
            if !audio::is_valid_sound(sound_id) {
                return Response::Error {
                    code: ErrorCode::UnknownSound,
                };
            }
            bt.stop_playback();
            bt.arm_play_stream(play_id);
            *pending_play = Some(PendingPlay {
                play_id,
                sound_id,
                offset: 0,
            });
            log::info!("playing sound {sound_id} play_id={play_id}");
            Response::Ok
        }

        (Opcode::PlayCancel, Request::PlayCancel { play_id: _ }) => {
            *pending_play = None;
            bt.stop_playback();
            Response::Ok
        }

        _ => Response::Error {
            code: ErrorCode::InvalidPayload,
        },
    }
}
