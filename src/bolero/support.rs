// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use bolero::{Driver, TypeGenerator, ValueGenerator};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ops::Bound;

pub fn gen_from_chars<D: Driver>(
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
            Ipv4Addr::from(d.gen_u32(
                Bound::Included(&0x1000_0000_u32),
                Bound::Excluded(&0xffff_ffff_u32),
            )?)
            .to_string(),
        ))
    }
}

pub struct Ipv6AddrString(pub String);

impl TypeGenerator for Ipv6AddrString {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(Ipv6AddrString(
            Ipv6Addr::from(d.gen_u128(
                Bound::Included(&0x0000_0000_0000_0000_0000_0000_0000_0001_u128),
                Bound::Excluded(&0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff_u128),
            )?)
            .to_string(),
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

mod unstable_unbounded_shift {
    // Unbounded shift is unstable in Rust 1.86.0
    // Remove when we upgrade to Rust 1.87.0
    // We should get an unused trait error with 1.87.0
    pub(crate) trait UnboundedShift {
        fn unbounded_shl(self, rhs: u32) -> Self;
        fn unbounded_shr(self, rhs: u32) -> Self;
    }

    impl UnboundedShift for u32 {
        fn unbounded_shl(self, amt: u32) -> Self {
            let (ret, overflow) = self.overflowing_shl(amt);
            if overflow { 0 } else { ret }
        }

        fn unbounded_shr(self, amt: u32) -> Self {
            let (ret, overflow) = self.overflowing_shr(amt);
            if overflow { 0 } else { ret }
        }
    }

    impl UnboundedShift for u128 {
        fn unbounded_shl(self, amt: u32) -> Self {
            let (ret, overflow) = self.overflowing_shl(amt);
            if overflow { 0 } else { ret }
        }

        fn unbounded_shr(self, amt: u32) -> Self {
            let (ret, overflow) = self.overflowing_shr(amt);
            if overflow { 0 } else { ret }
        }
    }
}
use unstable_unbounded_shift::UnboundedShift;

pub struct V4CidrString(pub String);
pub struct V6CidrString(pub String);
pub struct CidrString(pub String);

fn v4cdir_from_bytes(addr_bytes: u32, mask: u8) -> String {
    // Remove this allow once we upgrade to Rust 1.87.0
    #[allow(unstable_name_collisions)]
    let and_mask = u32::MAX.unbounded_shl(32 - u32::from(mask));
    let addr = Ipv4Addr::from(addr_bytes & and_mask);
    format!("{addr}/{mask}")
}

fn v6cdir_from_bytes(addr_bytes: u128, mask: u8) -> String {
    // Remove this allow once we upgrade to Rust 1.87.0
    #[allow(unstable_name_collisions)]
    let and_mask = u128::MAX.unbounded_shl(128 - u32::from(mask));
    let addr = Ipv6Addr::from(addr_bytes & and_mask);
    format!("{addr}/{mask}")
}

impl TypeGenerator for V4CidrString {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let mask = d.gen_u8(Bound::Included(&0), Bound::Included(&32))?;
        let addr_bytes = d.produce::<u32>()?;
        Some(V4CidrString(v4cdir_from_bytes(addr_bytes, mask)))
    }
}

impl TypeGenerator for V6CidrString {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let mask: u8 = d.gen_u8(Bound::Included(&0), Bound::Included(&128))?;
        let addr_bytes = d.produce::<u128>()?;
        Some(V6CidrString(v6cdir_from_bytes(addr_bytes, mask)))
    }
}

#[derive(Debug)]
pub struct UniqueV4CidrGenerator {
    count: u16,
    mask: u8,
}

impl UniqueV4CidrGenerator {
    #[must_use]
    pub fn new(count: u16, mask: u8) -> Self {
        Self { count, mask }
    }
}

impl ValueGenerator for UniqueV4CidrGenerator {
    // Remove this allow once we upgrade to Rust 1.87.0
    #![allow(unstable_name_collisions)]
    type Output = Vec<String>;

    fn generate<D: Driver>(&self, d: &mut D) -> Option<Self::Output> {
        if self.mask == 0 && self.count > 0 {
            d.produce::<u32>(); // generate a value to satisfiy the bolero driver
            return Some(vec!["0.0.0.0/0".to_string()]);
        }

        let available_addrs = 1_u32.unbounded_shl(u32::from(self.mask));
        let max_to_generate = if available_addrs > 0 {
            // Unwrap should never fail here because count is u16 and we take the min
            // The - 1 is to discount the 0 address which we won't generate
            #[allow(clippy::unwrap_used)]
            u16::try_from((available_addrs - 1).min(u32::from(self.count))).unwrap()
        } else {
            self.count
        };

        let addr_bytes_seed = d.gen_u32(
            Bound::Included(&0x1000_0000_u32),
            Bound::Included(&u32::MAX),
        )?;
        let mut cidrs = Vec::with_capacity(usize::from(self.count));
        let mut addrs_left = max_to_generate;
        let mut addr_bytes = addr_bytes_seed.unbounded_shr(u32::from(32 - self.mask));
        let addr_bytes_mask = u32::MAX.unbounded_shr(u32::from(32 - self.mask));
        while addrs_left > 0 {
            if addr_bytes & addr_bytes_mask == 0 {
                // Smallest valid v4 address with given mask
                addr_bytes = 1;
            }
            let cidr = v4cdir_from_bytes(
                addr_bytes.unbounded_shl(u32::from(32 - self.mask)),
                self.mask,
            );
            cidrs.push(cidr);
            addrs_left -= 1;
            addr_bytes = addr_bytes.wrapping_add(1);
        }
        Some(cidrs)
    }
}

#[derive(Debug)]
pub struct UniqueV6CidrGenerator {
    pub count: u16,
    pub mask: u8,
}

impl UniqueV6CidrGenerator {
    #[must_use]
    pub fn new(count: u16, mask: u8) -> Self {
        Self { count, mask }
    }
}

impl ValueGenerator for UniqueV6CidrGenerator {
    // Remove this allow once we upgrade to Rust 1.87.0
    #![allow(unstable_name_collisions)]
    type Output = Vec<String>;

    fn generate<D: Driver>(&self, d: &mut D) -> Option<Self::Output> {
        if self.mask == 0 && self.count > 0 {
            d.produce::<u32>(); // generate a value to satisfiy the bolero driver
            return Some(vec!["::/0".to_string()]);
        }

        let available_addrs = 1_u128.unbounded_shl(u32::from(self.mask));

        let max_to_generate = if available_addrs > 0 {
            // Unwrap should never fail here because count is u16 and we take the min
            // The - 1 is to discount the 0 address which we won't generate
            #[allow(clippy::unwrap_used)]
            u16::try_from((available_addrs - 1).min(u128::from(self.count))).unwrap()
        } else {
            self.count
        };

        let addr_bytes_seed = d.gen_u128(Bound::Included(&1_u128), Bound::Included(&u128::MAX))?;
        let mut cidrs = Vec::with_capacity(usize::from(self.count));
        let mut addrs_left = max_to_generate;
        let mut addr_bytes = addr_bytes_seed.unbounded_shr(u32::from(128 - self.mask));
        let addr_bytes_mask = u128::MAX.unbounded_shr(u32::from(128 - self.mask));
        while addrs_left > 0 {
            if addr_bytes & addr_bytes_mask == 0 {
                // Smallest valid v6 address with mask
                addr_bytes = 1;
            }
            let cidr = v6cdir_from_bytes(
                addr_bytes.unbounded_shl(u32::from(128 - self.mask)),
                self.mask,
            );
            cidrs.push(cidr);
            addrs_left -= 1;
            addr_bytes = addr_bytes.wrapping_add(1);
        }
        Some(cidrs)
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

pub const ALPHA_NUMERIC_CHARS: &str =
    "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub struct MacAddrString(pub String);
// Only generate lower case hex characters for mac addresses
// because we cannot customize PartialEq for generated types
// that use this, and we want to be able to compare generated
// mac addresses with each other without concern for case.
impl TypeGenerator for MacAddrString {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        // Generate a random 48-bit MAC address
        let mac = d.gen_u64(Bound::Included(&1), Bound::Excluded(&0xffff_ffff_ffff_u64))?;
        let bytes = [
            (mac & 0xff) as u8,
            ((mac >> 8) & 0xff) as u8,
            ((mac >> 16) & 0xff) as u8,
            ((mac >> 24) & 0xff) as u8,
            ((mac >> 32) & 0xff) as u8,
            ((mac >> 40) & 0xff) as u8,
        ];
        let mac_str = format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
        );
        Some(MacAddrString(mac_str))
    }
}

pub struct LinuxIfName(pub String);
const IF_NAME_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_";
const IF_NAME_MAX_LEN: usize = 16;

impl TypeGenerator for LinuxIfName {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        gen_from_chars(
            d,
            IF_NAME_CHARS,
            Bound::Included(&1),
            Bound::Included(&IF_NAME_MAX_LEN),
        )
        .map(LinuxIfName)
    }
}

pub struct LinuxIfNamesGenerator {
    pub count: u16,
}

impl ValueGenerator for LinuxIfNamesGenerator {
    type Output = Vec<String>;

    fn generate<D: Driver>(&self, d: &mut D) -> Option<Self::Output> {
        let ifnames = (0..self.count)
            .map(|i| {
                Some(format!(
                    "{}{i}",
                    gen_from_chars(
                        d,
                        IF_NAME_CHARS,
                        Bound::Included(&1),
                        Bound::Included(&(IF_NAME_MAX_LEN - 8)),
                    )?
                ))
            })
            .collect::<Option<Vec<_>>>()?;
        Some(ifnames)
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

#[cfg(test)]
mod test {
    #[test]
    fn test_unique_v4_cidr_generator() {
        for mask in 0..=32 {
            let generator = crate::bolero::support::UniqueV4CidrGenerator::new(10, mask);
            bolero::check!()
                .with_generator(generator)
                .with_iterations(1000) // Takes too long with auto-iterations
                .for_each(|cidrs| {
                    let mut seen = std::collections::HashSet::new();
                    for cidr in cidrs {
                        assert!(seen.insert(cidr), "Duplicate CIDR found: {cidr}");
                    }
                    assert!(
                        !cidrs.is_empty(),
                        "No CIDRs generated for mask={mask}, count=10"
                    );
                    assert!(cidrs.iter().all(|cidr| {
                        let (ip, mask) = cidr.split_once('/').unwrap();
                        assert!(mask.parse::<u8>().unwrap() <= 32);
                        ip.parse::<std::net::Ipv4Addr>().is_ok()
                    }));
                });
        }
    }

    #[test]
    fn test_unique_v6_cidr_generator() {
        for mask in 0..=128 {
            let generator = crate::bolero::support::UniqueV6CidrGenerator::new(10, mask);
            bolero::check!()
                .with_generator(generator)
                .with_iterations(1000) // Takes too long with auto-iterations
                .for_each(|cidrs| {
                    let mut seen = std::collections::HashSet::new();
                    assert!(
                        !cidrs.is_empty(),
                        "No CIDRs generated for mask={mask}, count=10"
                    );
                    for cidr in cidrs {
                        assert!(seen.insert(cidr), "Duplicate CIDR found: {cidr}");
                    }
                    assert!(cidrs.iter().all(|cidr| {
                        let (ip, mask) = cidr.split_once('/').unwrap();
                        assert!(mask.parse::<u8>().unwrap() <= 128);
                        ip.parse::<std::net::Ipv6Addr>().is_ok()
                    }));
                });
        }
    }
}
