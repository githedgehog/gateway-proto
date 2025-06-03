// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::bolero::support::CidrString;
use crate::config::{PeeringIPs, peering_i_ps as config_peering_i_ps};
use bolero::{Driver, TypeGenerator};

impl TypeGenerator for config_peering_i_ps::Rule {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let not = d.gen_bool(None)?;
        if not {
            Some(config_peering_i_ps::Rule::Not(d.produce::<CidrString>()?.0))
        } else {
            Some(config_peering_i_ps::Rule::Cidr(
                d.produce::<CidrString>()?.0,
            ))
        }
    }
}

pub mod peering_i_ps {
    use crate::bolero::support::{V4CidrString, V6CidrString};
    use crate::config::peering_i_ps::Rule;
    use bolero::{Driver, TypeGenerator};
    pub struct V4Rule(pub Rule);
    pub struct V6Rule(pub Rule);

    impl TypeGenerator for V4Rule {
        fn generate<D: Driver>(d: &mut D) -> Option<Self> {
            let not = d.gen_bool(None)?;
            let cidr = d.produce::<V4CidrString>()?.0;
            if not {
                Some(V4Rule(Rule::Not(cidr)))
            } else {
                Some(V4Rule(Rule::Cidr(cidr)))
            }
        }
    }

    impl TypeGenerator for V6Rule {
        fn generate<D: Driver>(d: &mut D) -> Option<Self> {
            let not = d.gen_bool(None)?;
            let cidr = d.produce::<V6CidrString>()?.0;
            if not {
                Some(V6Rule(Rule::Not(cidr)))
            } else {
                Some(V6Rule(Rule::Cidr(cidr)))
            }
        }
    }
}

impl TypeGenerator for PeeringIPs {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(PeeringIPs {
            // rule should never be None?
            rule: Some(d.produce::<config_peering_i_ps::Rule>()?),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct V4PeeringIPs(pub PeeringIPs);

#[derive(Debug, PartialEq, Clone)]
pub struct V6PeeringIPs(pub PeeringIPs);

impl TypeGenerator for V4PeeringIPs {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(V4PeeringIPs(PeeringIPs {
            rule: Some(d.produce::<peering_i_ps::V4Rule>()?.0),
        }))
    }
}

impl TypeGenerator for V6PeeringIPs {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(V6PeeringIPs(PeeringIPs {
            rule: Some(d.produce::<peering_i_ps::V6Rule>()?.0),
        }))
    }
}

#[cfg(test)]
mod test {
    use std::net::IpAddr;

    use crate::bolero::test_support::parse_cidr;
    use crate::bolero::{V4PeeringIPs, V6PeeringIPs};
    use crate::config::{PeeringIPs, peering_i_ps as config_peering_i_ps};

    #[test]
    fn test_peering_ips() {
        bolero::check!()
            .with_type::<PeeringIPs>()
            .for_each(|peering_ips: &PeeringIPs| {
                assert!(peering_ips.rule.is_some());
                let rule = peering_ips.rule.as_ref().unwrap();
                match rule {
                    config_peering_i_ps::Rule::Cidr(cidr)
                    | config_peering_i_ps::Rule::Not(cidr) => {
                        assert!(parse_cidr(cidr).is_ok());
                    }
                }
            });
    }

    fn test_peering_ip_rule<T>(rule: &config_peering_i_ps::Rule, test: T)
    where
        T: FnOnce(IpAddr, u8),
    {
        let (ip, mask) = match rule {
            config_peering_i_ps::Rule::Cidr(cidr) | config_peering_i_ps::Rule::Not(cidr) => {
                parse_cidr(cidr).unwrap()
            }
        };
        test(ip, mask);
    }

    #[test]
    fn test_v4_peering_ips() {
        bolero::check!()
            .with_type::<V4PeeringIPs>()
            .for_each(|v4_peering_ips: &V4PeeringIPs| {
                assert!(v4_peering_ips.0.rule.is_some());
                let rule = v4_peering_ips.0.rule.as_ref().unwrap();
                test_peering_ip_rule(rule, |ip, mask| {
                    assert!(ip.is_ipv4());
                    assert!(mask <= 32);
                });
            });
    }

    #[test]
    fn test_v6_peering_ips() {
        bolero::check!()
            .with_type::<V6PeeringIPs>()
            .for_each(|v6_peering_ips: &V6PeeringIPs| {
                assert!(v6_peering_ips.0.rule.is_some());
                let rule = v6_peering_ips.0.rule.as_ref().unwrap();
                test_peering_ip_rule(rule, |ip, mask| {
                    assert!(ip.is_ipv6());
                    assert!(mask <= 128);
                });
            });
    }
}
