use std::net::{Ipv4Addr, Ipv6Addr};

/// IPv4 embedded in an IPv4-mapped IPv6 address (`::ffff:a.b.c.d`), as lwIP often reports on ESP-IDF.
pub fn ipv4_from_mapped_addr_bytes(addr: [u8; 16]) -> Option<Ipv4Addr> {
    if addr[10] == 0xff && addr[11] == 0xff {
        Some(Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]))
    } else {
        None
    }
}

pub fn ipv4_from_ipv6(v6: Ipv6Addr) -> Option<Ipv4Addr> {
    v6.to_ipv4_mapped()
        .or_else(|| ipv4_from_mapped_addr_bytes(v6.octets()))
}

/// lwIP on ESP-IDF often stores an IPv4 peer in `sin6_addr.un.u32_addr[3]` on IPv6 httpd sockets.
pub fn ipv4_from_lwip_sin6(u8_addr: [u8; 16], u32_slot3: u32) -> Option<Ipv4Addr> {
    ipv4_from_mapped_addr_bytes(u8_addr)
        .or_else(|| ipv4_from_ipv6(Ipv6Addr::from(u8_addr)))
        .or_else(|| {
            let ip = Ipv4Addr::from(u32::from_be(u32_slot3));
            if ip.is_unspecified() {
                None
            } else {
                Some(ip)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lwip_u32_slot_ipv4() {
        let expected = Ipv4Addr::new(192, 168, 4, 2);
        let ip = ipv4_from_lwip_sin6([0u8; 16], u32::from(expected));
        assert_eq!(ip, Some(expected));
    }

    #[test]
    fn mapped_ipv6_bytes() {
        let mut bytes = [0u8; 16];
        bytes[10] = 0xff;
        bytes[11] = 0xff;
        bytes[12] = 192;
        bytes[13] = 168;
        bytes[14] = 4;
        bytes[15] = 22;
        assert_eq!(
            ipv4_from_mapped_addr_bytes(bytes),
            Some(Ipv4Addr::new(192, 168, 4, 22))
        );
    }
}
