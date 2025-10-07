// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use bolero::{Driver, TypeGenerator, ValueGenerator};
use std::ops::Bound;

use crate::bolero::support::{UniqueV4CidrGenerator, UniqueV6CidrGenerator};
use crate::config::expose::Nat;
use crate::config::{
    Expose, PeeringAs, PeeringIPs, PeeringStatefulNat, PeeringStatelessNat, peering_as,
    peering_i_ps,
};
use crate::google::protobuf::Duration;

struct UniquePeeringAs<T: ValueGenerator<Output = Vec<String>>> {
    cidr_producer: T,
}

impl<T: ValueGenerator<Output = Vec<String>>> UniquePeeringAs<T> {
    pub fn new(cidr_producer: T) -> Self {
        Self { cidr_producer }
    }
}

impl<T> ValueGenerator for UniquePeeringAs<T>
where
    T: ValueGenerator<Output = Vec<String>>,
{
    type Output = Vec<PeeringAs>;

    fn generate<D: Driver>(&self, d: &mut D) -> Option<Self::Output> {
        let cidrs = self.cidr_producer.generate(d)?;
        let r#as = (0..cidrs.len())
            .map(|i| {
                let use_not = d.gen_bool(None)?;
                let cidr = cidrs[i].clone();
                let rule = if use_not {
                    peering_as::Rule::Not(cidr)
                } else {
                    peering_as::Rule::Cidr(cidr)
                };
                Some(PeeringAs { rule: Some(rule) })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(r#as)
    }
}

struct UniquePeeringIPs<T: ValueGenerator<Output = Vec<String>>> {
    cidr_producer: T,
}

impl<T: ValueGenerator<Output = Vec<String>>> UniquePeeringIPs<T> {
    pub fn new(cidr_producer: T) -> Self {
        Self { cidr_producer }
    }
}

impl<T> ValueGenerator for UniquePeeringIPs<T>
where
    T: ValueGenerator<Output = Vec<String>>,
{
    type Output = Vec<PeeringIPs>;

    fn generate<D: Driver>(&self, d: &mut D) -> Option<Self::Output> {
        let cidrs = self.cidr_producer.generate(d)?;
        let ips = (0..cidrs.len())
            .map(|i| {
                let use_not = d.gen_bool(None)?;
                let cidr = cidrs[i].clone();
                let rule = if use_not {
                    peering_i_ps::Rule::Not(cidr)
                } else {
                    peering_i_ps::Rule::Cidr(cidr)
                };
                Some(PeeringIPs { rule: Some(rule) })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(ips)
    }
}

impl TypeGenerator for PeeringStatefulNat {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let seconds = d.gen_i64(Bound::Included(&0), Bound::Included(&i64::MAX))?;
        let nanos = d.gen_i32(Bound::Included(&0), Bound::Included(&999_999_999))?;
        let idle_timeout = Some(Duration { seconds, nanos });
        Some(PeeringStatefulNat { idle_timeout })
    }
}

impl TypeGenerator for PeeringStatelessNat {
    fn generate<D: Driver>(_d: &mut D) -> Option<Self> {
        Some(PeeringStatelessNat {})
    }
}

impl TypeGenerator for Nat {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let stateful = d.gen_bool(None)?;
        if stateful {
            Some(Nat::Stateful(d.produce()?))
        } else {
            Some(Nat::Stateless(d.produce()?))
        }
    }
}

// FIXME(manishv): We should make sure that the number of peering ips and ases are
// consistent.
// FIXME(manishv): We should also make sure that the cidrs use not
impl TypeGenerator for Expose {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let v4 = d.gen_bool(None)?;
        let len = d.gen_u16(Bound::Included(&1), Bound::Included(&10))?;
        let v4_mask: u8 = d.gen_u8(Bound::Included(&8), Bound::Included(&32))?;
        let v6_mask: u8 = d.gen_u8(Bound::Included(&16), Bound::Included(&128))?;

        let peering_ips = if v4 {
            let v4_cidr_producer_ips =
                UniquePeeringIPs::new(UniqueV4CidrGenerator::new(len, v4_mask));
            v4_cidr_producer_ips.generate(d)?
        } else {
            let v6_cidr_producer_ips =
                UniquePeeringIPs::new(UniqueV6CidrGenerator::new(len, v6_mask));
            v6_cidr_producer_ips.generate(d)?
        };

        let has_as = d.gen_bool(None)?;
        let (r#as, nat) = if has_as {
            let nat = d.produce()?;
            if v4 {
                let v4_cidr_producer_as =
                    UniquePeeringAs::new(UniqueV4CidrGenerator::new(len, v4_mask));
                (v4_cidr_producer_as.generate(d)?, Some(nat))
            } else {
                let v6_cidr_producer_as =
                    UniquePeeringAs::new(UniqueV6CidrGenerator::new(len, v6_mask));
                (v6_cidr_producer_as.generate(d)?, Some(nat))
            }
        } else {
            (vec![], None)
        };

        Some(Expose {
            ips: peering_ips,
            r#as,
            nat,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bolero::test_support::parse_cidr;
    use crate::bolero::test_support::{get_peering_as_ip, get_peering_ip};
    use crate::config::expose;
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
                (IpAddr::V4(_), &IpAddrType::V6) | (IpAddr::V6(_), &IpAddrType::V4) => {
                    return false;
                }
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
                assert!(!expose.ips.is_empty());
                if expose.ips.len() > 1 {
                    more_than_one = true;
                }
                assert!(ip_type_same(
                    expose
                        .ips
                        .iter()
                        .map(|ip| parse_cidr(get_peering_ip(ip).unwrap()).unwrap().0)
                        .collect::<Vec<_>>()
                        .as_slice()
                ));
                assert!(ip_type_same(
                    expose
                        .r#as
                        .iter()
                        .map(|r#as| parse_cidr(get_peering_as_ip(r#as).unwrap()).unwrap().0)
                        .collect::<Vec<_>>()
                        .as_slice()
                ));
                if !expose.r#as.is_empty() {
                    let Some(nat) = expose.nat else {
                        panic!("nat is None");
                    };
                    match nat {
                        expose::Nat::Stateless(_s) => {}
                        expose::Nat::Stateful(s) => {
                            assert!(s.idle_timeout.is_some());
                        }
                    }
                }
            });
        assert!(more_than_one);
    }
}
