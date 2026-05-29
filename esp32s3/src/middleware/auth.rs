use std::{mem, net::Ipv4Addr, sync::OnceLock};

use anyhow::Context;
use domination_auth::{AuthToken, NowFn, RngFn, UserManager};
use esp_idf_svc::{
    handle::RawHandle,
    http::server::Request,
    sys::{
        esp_random, httpd_req_to_sockfd, httpd_req_t, lwip_getpeername, sockaddr_in, sockaddr_in6,
        AF_INET, AF_INET6,
    },
};

use crate::{
    hardware::network,
    http::RequestContext,
    middleware::client_ip::ipv4_from_lwip_sin6,
};

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

fn usable_ipv4(ip: Ipv4Addr) -> Option<Ipv4Addr> {
    if ip.is_unspecified() {
        None
    } else {
        Some(ip)
    }
}

/// ESP-IDF httpd sockets are often IPv6; read peer with `sockaddr_in6` then extract IPv4.
unsafe fn peer_ipv4_from_httpd(req: *mut httpd_req_t) -> Option<Ipv4Addr> {
    let sockfd = httpd_req_to_sockfd(req);
    if sockfd == -1 {
        return None;
    }

    let mut addr: sockaddr_in6 = mem::zeroed();
    let mut addrlen = mem::size_of::<sockaddr_in6>() as u32;
    if lwip_getpeername(
        sockfd,
        &mut addr as *mut _ as *mut _,
        &mut addrlen as *mut _,
    ) != 0
    {
        return None;
    }

    if addr.sin6_family == AF_INET as u8 {
        let addr4: &sockaddr_in = &*(&addr as *const sockaddr_in6 as *const sockaddr_in);
        return usable_ipv4(Ipv4Addr::from(u32::from_be(addr4.sin_addr.s_addr)));
    }

    if addr.sin6_family == AF_INET6 as u8 {
        let bytes = addr.sin6_addr.un.u8_addr;
        let slot3 = addr.sin6_addr.un.u32_addr[3];
        return ipv4_from_lwip_sin6(bytes, slot3).and_then(usable_ipv4);
    }

    None
}

/// Client IPv4 for auth token binding.
///
/// - **STA (joins existing Wi‑Fi):** peer is the phone/PC on the same LAN as the ESP.
/// - **SoftAP (ESP is the AP):** peer is the station on the ESP subnet (e.g. 192.168.4.x).
pub fn client_ipv4(
    req: &mut Request<&mut esp_idf_svc::http::server::EspHttpConnection>,
) -> anyhow::Result<String> {
    let raw = req
        .connection()
        .raw_connection()
        .context("Falha de autenticacao")?;

    #[cfg(esp_idf_lwip_ipv4)]
    if let Ok(ip) = raw.source_ipv4() {
        if let Some(ip) = usable_ipv4(ip) {
            return Ok(ip.to_string());
        }
    }

    if let Some(ip) = unsafe { peer_ipv4_from_httpd(raw.handle()) } {
        return Ok(ip.to_string());
    }

    #[cfg(esp_idf_lwip_ipv6)]
    if let Ok(v6) = raw.source_ipv6() {
        if let Some(ip) = crate::middleware::client_ip::ipv4_from_ipv6(v6).and_then(usable_ipv4) {
            return Ok(ip.to_string());
        }
    }

    if let Some(ip) = unsafe { network::softap_peer_ipv4_from_dhcp() } {
        return Ok(ip.to_string());
    }

    log::warn!(
        "could not resolve client ipv4 (topology={})",
        network::topology_label()
    );
    Err(anyhow::anyhow!("Falha de autenticacao"))
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

