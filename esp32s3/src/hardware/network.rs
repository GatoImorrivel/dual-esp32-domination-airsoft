use std::{
    net::Ipv4Addr,
    sync::atomic::{AtomicPtr, AtomicU8, Ordering},
};

use esp_idf_svc::sys::{
    esp_netif_dhcps_get_clients_by_mac, esp_netif_pair_mac_ip_t, esp_netif_t,
    esp_wifi_ap_get_sta_list, ESP_OK, wifi_sta_list_t,
};

/// 0 = unknown, 1 = SoftAP (ESP is the AP), 2 = STA (ESP joins existing Wi‑Fi).
static TOPOLOGY: AtomicU8 = AtomicU8::new(0);
static AP_NETIF: AtomicPtr<esp_netif_t> = AtomicPtr::new(std::ptr::null_mut());

pub fn init_netifs(ap: *mut esp_netif_t, _sta: *mut esp_netif_t) {
    AP_NETIF.store(ap, Ordering::Relaxed);
}

pub fn set_softap_topology() {
    TOPOLOGY.store(1, Ordering::Relaxed);
}

pub fn set_station_topology() {
    TOPOLOGY.store(2, Ordering::Relaxed);
}

pub fn topology_label() -> &'static str {
    match TOPOLOGY.load(Ordering::Relaxed) {
        1 => "softap",
        2 => "sta",
        _ => "unknown",
    }
}

pub fn is_softap_topology() -> bool {
    TOPOLOGY.load(Ordering::Relaxed) == 1
}

fn ipv4_from_esp_ip4(addr: u32) -> Option<Ipv4Addr> {
    let ip = Ipv4Addr::from(u32::from_be(addr));
    if ip.is_unspecified() {
        None
    } else {
        Some(ip)
    }
}

/// When lwIP does not fill the TCP peer on the AP interface, resolve the station IP from the
/// SoftAP DHCP lease table. Safe only with a single connected station (typical field setup).
pub unsafe fn softap_peer_ipv4_from_dhcp() -> Option<Ipv4Addr> {
    if !is_softap_topology() {
        return None;
    }

    let ap_netif = AP_NETIF.load(Ordering::Relaxed);
    if ap_netif.is_null() {
        return None;
    }

    let mut sta_list = wifi_sta_list_t::default();
    if esp_wifi_ap_get_sta_list(&mut sta_list) != ESP_OK {
        return None;
    }

    let count = sta_list.num;
    if count <= 0 {
        return None;
    }
    if count != 1 {
        log::warn!(
            "softap dhcp ip fallback: {count} stations connected, using first station"
        );
    }

    let mut pair = esp_netif_pair_mac_ip_t {
        mac: sta_list.sta[0].mac,
        ip: Default::default(),
    };
    if esp_netif_dhcps_get_clients_by_mac(ap_netif, 1, &mut pair) != ESP_OK {
        return None;
    }

    ipv4_from_esp_ip4(pair.ip.addr)
}
