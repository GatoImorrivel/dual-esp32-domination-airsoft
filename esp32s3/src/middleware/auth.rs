use std::sync::OnceLock;

use anyhow::Context;
use domination_auth::{AuthToken, NowFn, RngFn, UserManager};
use esp_idf_svc::http::server::Request;
use esp_idf_svc::sys::esp_random;

use crate::http::RequestContext;

static USER_MANAGER: OnceLock<UserManager> = OnceLock::new();

fn esp_rng(buf: &mut [u8]) {
    for chunk in buf.chunks_mut(4) {
        let r = unsafe { esp_random() };
        chunk.copy_from_slice(&r.to_le_bytes());
    }
}

fn system_now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

fn load_user_manager() -> UserManager {
    UserManager::new(
        vec![(
            env!("ADMIN_USER").to_string(),
            env!("ADMIN_PASS").to_string(),
        )],
        esp_rng as RngFn,
        system_now_secs as NowFn,
    )
}

pub fn user_manager() -> UserManager {
    USER_MANAGER
        .get_or_init(load_user_manager)
        .clone()
}

/// SoftAP mode often reports 0.0.0.0 from lwIP; use a stable sentinel for session binding.
pub fn normalize_client_ip(ip: std::net::Ipv4Addr) -> String {
    if ip.is_unspecified() || ip.is_loopback() {
        "lan".to_string()
    } else {
        ip.to_string()
    }
}

pub fn client_ipv4(
    req: &mut Request<&mut esp_idf_svc::http::server::EspHttpConnection>,
) -> anyhow::Result<String> {
    let ip = req
        .connection()
        .raw_connection()
        .context("Falha de autenticacao")?
        .source_ipv4()
        .map_err(|_| anyhow::anyhow!("Falha de autenticacao"))?;
    Ok(normalize_client_ip(ip))
}

pub fn check_admin_auth(ctx: &RequestContext) -> anyhow::Result<()> {
    let client_ip = ctx
        .client_ip
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Não autorizado"))?;

    let token_str = ctx
        .authorization
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Não autorizado"))?;

    let now = system_now_secs();
    let parsed = AuthToken::from_string(token_str, now)
        .ok_or_else(|| anyhow::anyhow!("Token invalido"))?;

    if parsed.ip() != client_ip {
        log::warn!(
            "auth ip mismatch token_ip={} client_ip={}",
            parsed.ip(),
            client_ip
        );
        return Err(anyhow::anyhow!("Não autorizado"));
    }

    let mgr = user_manager();
    if mgr.validate_token(&parsed, client_ip) {
        return Ok(());
    }

    log::warn!("auth token not current session for ip={}", client_ip);
    Err(anyhow::anyhow!("Não autorizado"))
}

