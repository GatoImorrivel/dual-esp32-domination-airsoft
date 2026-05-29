use anyhow::{anyhow, Result};

/// Format a 6-byte MAC as `AA:BB:CC:DD:EE:FF`.
pub fn format_mac(addr: [u8; 6]) -> String {
    addr.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":")
}

/// Parse `AA:BB:CC:DD:EE:FF` (case-insensitive, `:` or `-` separators).
pub fn parse_mac(s: &str) -> Result<[u8; 6]> {
    let hex: String = s
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect::<String>()
        .to_uppercase();
    if hex.len() != 12 {
        return Err(anyhow!("MAC must have 6 bytes, got {}", hex.len() / 2));
    }
    let mut addr = [0u8; 6];
    for (i, byte) in addr.iter_mut().enumerate() {
        let pair = &hex[i * 2..i * 2 + 2];
        *byte = u8::from_str_radix(pair, 16).map_err(|e| anyhow!("invalid hex: {e}"))?;
    }
    Ok(addr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_mac() {
        let addr = [0xAA, 0xBB, 0xCC, 0x11, 0x22, 0x33];
        let s = format_mac(addr);
        assert_eq!(parse_mac(&s).unwrap(), addr);
        assert_eq!(parse_mac("aa-bb-cc-11-22-33").unwrap(), addr);
    }
}
