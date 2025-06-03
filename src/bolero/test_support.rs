// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::config::{PeeringAs, PeeringIPs, peering_as, peering_i_ps};
use thiserror::Error;
#[derive(Debug, Error)]
pub enum CidrParseError {
    #[error("Unable to parse IP address: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),

    #[error("Invalid mask for IP: {0}, {1}")]
    MaskParseError(String, String),
    #[error("Invalid mask length for IP: {0}, {1}")]
    MaskLenError(u8, String),
}

/// Parse a CIDR string into an IP address and mask length.
///
/// # Errors
///
/// Returns an error if the CIDR string is invalid.
///
pub fn parse_cidr(cidr: &str) -> Result<(std::net::IpAddr, u8), CidrParseError> {
    let parts: Vec<&str> = cidr.split('/').collect();
    let ip = parts[0]
        .parse::<std::net::IpAddr>()
        .map_err(CidrParseError::AddrParseError)?;
    let mask = parts[1]
        .parse::<u8>()
        .map_err(|_| CidrParseError::MaskParseError(parts[1].to_string(), cidr.to_string()))?;
    if ip.is_ipv4() && mask > 32 {
        return Err(CidrParseError::MaskLenError(mask, cidr.to_string()));
    }
    if ip.is_ipv6() && mask > 128 {
        return Err(CidrParseError::MaskLenError(mask, cidr.to_string()));
    }

    Ok((ip, mask))
}

#[must_use]
pub fn get_peering_ip(item: &PeeringIPs) -> Option<&str> {
    match &item.rule {
        Some(peering_i_ps::Rule::Cidr(ip) | peering_i_ps::Rule::Not(ip)) => Some(ip),
        None => None,
    }
}

#[must_use]
pub fn get_peering_as_ip(item: &PeeringAs) -> Option<&str> {
    match &item.rule {
        Some(peering_as::Rule::Cidr(ip) | peering_as::Rule::Not(ip)) => Some(ip),
        None => None,
    }
}
