// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::bolero::support::CidrString;
use crate::config::{PeeringAs, peering_as as config_peering_as};
use bolero::{Driver, TypeGenerator};

impl TypeGenerator for PeeringAs {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(PeeringAs {
            rule: Some(d.produce::<config_peering_as::Rule>()?),
        })
    }
}

impl TypeGenerator for config_peering_as::Rule {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let not = d.gen_bool(None)?;
        if not {
            Some(config_peering_as::Rule::Not(d.produce::<CidrString>()?.0))
        } else {
            Some(config_peering_as::Rule::Cidr(d.produce::<CidrString>()?.0))
        }
    }
}

pub mod peering_as {
    use crate::bolero::support::{V4CidrString, V6CidrString};
    use crate::config::peering_as::Rule;
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

#[derive(Debug, PartialEq, Clone)]
pub struct V4PeeringAs(pub PeeringAs);

#[derive(Debug, PartialEq, Clone)]
pub struct V6PeeringAs(pub PeeringAs);

impl TypeGenerator for V4PeeringAs {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(V4PeeringAs(PeeringAs {
            rule: Some(d.produce::<peering_as::V4Rule>()?.0),
        }))
    }
}

impl TypeGenerator for V6PeeringAs {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(V6PeeringAs(PeeringAs {
            rule: Some(d.produce::<peering_as::V6Rule>()?.0),
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::bolero::test_support::parse_cidr;
    use crate::bolero::{V4PeeringAs, V6PeeringAs};
    use crate::config::{PeeringAs, peering_as as config_peering_as};
    use std::net::IpAddr;
    #[test]
    fn test_peering_as() {
        bolero::check!()
            .with_type::<PeeringAs>()
            .for_each(|peering_as: &PeeringAs| {
                assert!(peering_as.rule.is_some());
                let rule = peering_as.rule.as_ref().unwrap();
                match rule {
                    config_peering_as::Rule::Cidr(cidr) | config_peering_as::Rule::Not(cidr) => {
                        assert!(parse_cidr(cidr).is_ok());
                    }
                }
            });
    }

    fn test_peering_as_rule<T>(rule: &config_peering_as::Rule, test: T)
    where
        T: FnOnce(IpAddr, u8),
    {
        let (ip, mask) = match rule {
            config_peering_as::Rule::Cidr(cidr) | config_peering_as::Rule::Not(cidr) => {
                parse_cidr(cidr).unwrap()
            }
        };
        test(ip, mask);
    }

    #[test]
    fn test_v4_peering_as() {
        bolero::check!()
            .with_type::<V4PeeringAs>()
            .for_each(|v4_peering_as: &V4PeeringAs| {
                assert!(v4_peering_as.0.rule.is_some());
                let rule = v4_peering_as.0.rule.as_ref().unwrap();
                test_peering_as_rule(rule, |ip, mask| {
                    assert!(ip.is_ipv4());
                    assert!(mask <= 32);
                });
            });
    }

    #[test]
    fn test_v6_peering_as() {
        bolero::check!()
            .with_type::<V6PeeringAs>()
            .for_each(|v6_peering_as: &V6PeeringAs| {
                assert!(v6_peering_as.0.rule.is_some());
                let rule = v6_peering_as.0.rule.as_ref().unwrap();
                test_peering_as_rule(rule, |ip, mask| {
                    assert!(ip.is_ipv6());
                    assert!(mask <= 128);
                });
            });
    }
}
