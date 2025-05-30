// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

use crate::config::{IfType, Interface, OspfConfig, OspfInterface};
use bolero::{Driver, TypeGenerator};

use crate::bolero::support::{Ipv4AddrString, LinuxIfName, MacAddrString};

impl TypeGenerator for OspfInterface {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let area = d.produce::<Ipv4AddrString>()?.0;
        Some(OspfInterface {
            passive: d.produce()?,
            area, // Should this be Ipv4 or Ipv6 or a random integer?
            cost: d.produce()?,
            network_type: d.produce()?,
        })
    }
}

impl TypeGenerator for OspfConfig {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let router_id = d.produce::<Ipv4AddrString>()?.0;
        Some(OspfConfig {
            router_id,
            vrf: d.produce()?,
        })
    }
}

impl TypeGenerator for Interface {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let r#type: IfType = d.produce()?;
        let ipaddrs = if d.gen_bool(None)? {
            match r#type {
                IfType::Ethernet | IfType::Loopback | IfType::Vlan => {
                    let ipaddrs = d.produce::<Vec<Ipv4AddrString>>()?;
                    ipaddrs.into_iter().map(|v| v.0).collect()
                }
                IfType::Vtep => vec![d.produce::<Ipv4AddrString>()?.0],
            }
        } else {
            vec![]
        };

        let vlan = match r#type {
            IfType::Ethernet | IfType::Loopback | IfType::Vtep => None,
            IfType::Vlan => Some((u32::from(d.produce::<u16>()?)) & 0xfff), // 12 bits for VLAN ID
        };

        let macaddr = match r#type {
            IfType::Ethernet | IfType::Vlan => Some(d.produce::<MacAddrString>()?.0),
            _ => None,
        };

        let ospf = match r#type {
            IfType::Ethernet | IfType::Vlan => {
                if d.gen_bool(None)? {
                    Some(d.produce::<OspfInterface>()?)
                } else {
                    None
                }
            }
            _ => None,
        };

        Some(Interface {
            name: d.produce::<LinuxIfName>()?.0,
            ipaddrs,
            r#type: r#type.into(),
            role: d.produce()?,
            vlan,
            macaddr,
            ospf,
            system_name: None, // We do not support system names right now
        })
    }
}

#[cfg(test)]
mod test {
    use crate::config::{IfType, Interface, OspfConfig, OspfInterface};

    #[test]
    fn test_ospf_interface() {
        bolero::check!()
            .with_type::<OspfInterface>()
            .for_each(|intf: &OspfInterface| {
                assert!(intf.area.parse::<std::net::Ipv4Addr>().is_ok());
            });
    }

    #[test]
    fn test_ospf_config() {
        bolero::check!()
            .with_type::<OspfConfig>()
            .for_each(|config: &OspfConfig| {
                assert!(config.router_id.parse::<std::net::Ipv4Addr>().is_ok());
            });
    }

    #[test]
    fn test_interface() {
        bolero::check!()
            .with_type::<Interface>()
            .for_each(|intf: &Interface| {
                assert!(
                    intf.name.len() <= 16,
                    "Interface name too long: {} len: {}",
                    intf.name,
                    intf.name.len()
                );
                assert!(
                    intf.ipaddrs
                        .iter()
                        .all(|ip| ip.parse::<std::net::Ipv4Addr>().is_ok())
                );
                assert!(intf.macaddr.is_some() || intf.r#type != i32::from(IfType::Ethernet));
                assert!(intf.vlan.is_some() || intf.r#type != i32::from(IfType::Vlan));
                assert!(!intf.ospf.is_some() || intf.r#type != i32::from(IfType::Loopback));
                assert!(!intf.ospf.is_some() || intf.r#type != i32::from(IfType::Vtep));
                assert!(intf.system_name.is_none());
            });
    }
}
