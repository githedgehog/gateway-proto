// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

use bolero::{Driver, TypeGenerator};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ops::Bound;

fn gen_from_chars<D: Driver>(
    d: &mut D,
    chars: &str,
    min: std::ops::Bound<&usize>,
    max: std::ops::Bound<&usize>,
) -> Option<String> {
    let len = d.gen_usize(min, max)?;
    (0..len)
        .map(|_| {
            chars
                .chars()
                .nth(d.gen_usize(Bound::Included(&0), Bound::Excluded(&chars.len()))?)
        })
        .collect()
}

pub struct Ipv4AddrString(pub String);

impl TypeGenerator for Ipv4AddrString {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(Ipv4AddrString(
            Ipv4Addr::from(d.produce::<u32>()?).to_string(),
        ))
    }
}

pub struct Ipv6AddrString(pub String);

impl TypeGenerator for Ipv6AddrString {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(Ipv6AddrString(
            Ipv6Addr::from(d.produce::<u128>()?).to_string(),
        ))
    }
}

pub struct IpAddrString(pub String);

impl TypeGenerator for IpAddrString {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let is_ipv4 = d.gen_bool(None)?;
        if is_ipv4 {
            Some(IpAddrString(d.produce::<Ipv4AddrString>()?.0))
        } else {
            Some(IpAddrString(d.produce::<Ipv6AddrString>()?.0))
        }
    }
}

pub struct V4CidrString(pub String);
pub struct V6CidrString(pub String);
pub struct CidrString(pub String);

impl TypeGenerator for V4CidrString {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let addr = d.produce::<Ipv4AddrString>()?.0;
        let mask = d.gen_usize(Bound::Included(&0), Bound::Included(&32))?;
        Some(V4CidrString(format!("{addr}/{mask}")))
    }
}

impl TypeGenerator for V6CidrString {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let addr = d.produce::<Ipv6AddrString>()?.0;
        let mask = d.gen_usize(Bound::Included(&0), Bound::Included(&128))?;
        Some(V6CidrString(format!("{addr}/{mask}")))
    }
}

impl TypeGenerator for CidrString {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let is_ipv4 = d.gen_bool(None)?;
        if is_ipv4 {
            Some(CidrString(d.produce::<V4CidrString>()?.0))
        } else {
            Some(CidrString(d.produce::<V6CidrString>()?.0))
        }
    }
}

pub struct MacAddrString(pub String);
const LOWER_HEX_CHARS: &str = "0123456789abcdef";
// Only generate lower case hex characters for mac addresses
// because we cannot customize PartialEq for generated types
// that use this, and we want to be able to compare generated
// mac addresses with each other without concern for case.
impl TypeGenerator for MacAddrString {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let s: Option<String> = (0..6)
            .map(|_| gen_from_chars(d, LOWER_HEX_CHARS, Bound::Included(&2), Bound::Excluded(&3)))
            .collect::<Option<Vec<String>>>()
            .map(|v| v.join(":"));
        s.map(MacAddrString)
    }
}

pub struct LinuxIfName(pub String);
const IF_NAME_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_";

impl TypeGenerator for LinuxIfName {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        gen_from_chars(d, IF_NAME_CHARS, Bound::Included(&1), Bound::Included(&16)).map(LinuxIfName)
    }
}

pub struct K8sObjectNameString(pub String);

const K8S_END_CHAR: &str = "abcdefghijklmnopqrstuvwxyz0123456789";
const K8S_OTHER_CHARS: &str = "abcdefghijklmnopqrstuvwxyz0123456789-";
const K8S_OBJ_MAX_LEN: usize = 63;

impl TypeGenerator for K8sObjectNameString {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let len = d.gen_usize(Bound::Included(&2), Bound::Included(&K8S_OBJ_MAX_LEN))?;
        let first_char = gen_from_chars(d, K8S_END_CHAR, Bound::Included(&1), Bound::Included(&1))?;
        let middle_chars = gen_from_chars(
            d,
            K8S_OTHER_CHARS,
            Bound::Included(&0),
            Bound::Excluded(&(len - 2)),
        )?;
        let end_char = gen_from_chars(d, K8S_END_CHAR, Bound::Included(&0), Bound::Excluded(&1))?;

        Some(K8sObjectNameString(format!(
            "{first_char}{middle_chars}{end_char}"
        )))
    }
}

pub fn choose<T: Clone, D: Driver>(d: &mut D, choices: &[T]) -> Option<T> {
    let index = d.gen_usize(Bound::Included(&0), Bound::Excluded(&choices.len()))?;
    Some(choices[index].clone())
}
