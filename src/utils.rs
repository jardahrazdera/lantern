// src/utils.rs
#![allow(clippy::iter_nth_zero)] // .nth(0) is clearer in this context
use byte_unit::Byte;

#[allow(dead_code)]
pub fn format_bytes(bytes: u64) -> String {
    let byte = Byte::from_u128(bytes as u128).unwrap_or(Byte::from_u64(0));
    format!(
        "{:.2}",
        byte.get_appropriate_unit(byte_unit::UnitType::Binary)
    )
}

#[allow(dead_code)]
pub fn validate_ip(ip: &str) -> bool {
    ip.parse::<std::net::IpAddr>().is_ok()
        || ip
            .split('/')
            .nth(0)
            .and_then(|addr| addr.parse::<std::net::IpAddr>().ok())
            .is_some()
}

#[allow(dead_code)]
pub fn validate_gateway(gateway: &str) -> bool {
    gateway.parse::<std::net::IpAddr>().is_ok()
}
