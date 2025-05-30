// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

use bolero::{Driver, TypeGenerator};
use std::ops::Bound;

use crate::bolero::impl_peering_as::V4PeeringAs;
use crate::bolero::impl_peering_as::V6PeeringAs;
use crate::bolero::impl_peering_i_ps::V4PeeringIPs;
use crate::bolero::impl_peering_i_ps::V6PeeringIPs;
use crate::config::Expose;

// FIXME(manishv): We should make sure that the number of peering ips and ases are
// consistent.
impl TypeGenerator for Expose {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let v4 = d.gen_bool(None)?;
        let len = d.gen_usize(Bound::Included(&1), Bound::Included(&10))?;

        let peering_ips = (0..len)
            .map(|_| {
                let peering_ips = if v4 {
                    d.produce::<V4PeeringIPs>()?.0
                } else {
                    d.produce::<V6PeeringIPs>()?.0
                };
                Some(peering_ips)
            })
            .collect::<Option<Vec<_>>>()?;

        let has_as = d.gen_bool(None)?;
        let r#as = if has_as {
            (0..len)
                .map(|_| {
                    let r#as = if v4 {
                        d.produce::<V4PeeringAs>()?.0
                    } else {
                        d.produce::<V6PeeringAs>()?.0
                    };
                    Some(r#as)
                })
                .collect::<Option<Vec<_>>>()
        } else {
            Some(vec![])
        }?;

        Some(Expose {
            ips: peering_ips,
            r#as,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bolero::test_support::parse_cidr;
    use crate::bolero::test_support::{get_peering_as_ip, get_peering_ip};
    use std::net::IpAddr;

    enum IpAddrType {
        V4,
        V6,
        Unknown,
    }

    fn ip_type_same(ips: &[IpAddr]) -> bool {
        let mut ip_type = IpAddrType::Unknown;
        for ip in ips {
            match (ip, &ip_type) {
                (IpAddr::V4(_), &IpAddrType::Unknown) => ip_type = IpAddrType::V4,
                (IpAddr::V6(_), &IpAddrType::Unknown) => ip_type = IpAddrType::V6,
                (IpAddr::V4(_), &IpAddrType::V6) => return false,
                (IpAddr::V6(_), &IpAddrType::V4) => return false,
                _ => {}
            }
        }
        true
    }

    #[test]
    fn test_expose() {
        let mut more_than_one = false;
        bolero::check!()
            .with_type::<Expose>()
            .for_each(|expose: &Expose| {
                assert!(expose.ips.len() > 0);
                if expose.ips.len() > 1 {
                    more_than_one = true;
                }
                assert!(ip_type_same(
                    expose
                        .ips
                        .iter()
                        .map(|ip| parse_cidr(&get_peering_ip(ip).unwrap()).unwrap().0)
                        .collect::<Vec<_>>()
                        .as_slice()
                ));
                assert!(ip_type_same(
                    expose
                        .r#as
                        .iter()
                        .map(|r#as| parse_cidr(&get_peering_as_ip(r#as).unwrap()).unwrap().0)
                        .collect::<Vec<_>>()
                        .as_slice()
                ));
            });
        assert!(more_than_one);
    }
}
