// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::bolero::support::{
    Ipv4AddrString, LinuxIfName, MacAddrString, UniqueV4InterfaceAddressGenerator,
    UniqueV6InterfaceAddressGenerator,
};
use crate::config::{IfRole, IfType, Interface, OspfConfig, OspfInterface, OspfNetworkType};
use bolero::{Driver, TypeGenerator, ValueGenerator};
use std::ops::Bound;

impl TypeGenerator for OspfInterface {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let area = d.produce::<Ipv4AddrString>()?.0;
        Some(OspfInterface {
            passive: d.produce()?,
            area, // Should this be Ipv4 or Ipv6 or a random integer?
            cost: d.produce()?,
            network_type: Some(d.produce::<OspfNetworkType>()?.into()),
        })
    }
}

impl TypeGenerator for OspfConfig {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let router_id = d.produce::<Ipv4AddrString>()?.0;
        Some(OspfConfig {
            router_id,
            vrf: if d.gen_bool(None)? {
                Some(d.produce::<LinuxIfName>()?.0)
            } else {
                None
            },
        })
    }
}

impl TypeGenerator for Interface {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let r#type: IfType = d.produce()?;
        let ipaddrs = if d.gen_bool(None)? || r#type == IfType::Vtep {
            match r#type {
                IfType::Ethernet | IfType::Loopback | IfType::Vlan => {
                    let count_v4 = d.gen_u16(Bound::Included(&0), Bound::Included(&10))?;
                    let count_v6 = d.gen_u16(Bound::Included(&0), Bound::Included(&10))?;
                    let addrs_v4 = UniqueV4InterfaceAddressGenerator::new(count_v4).generate(d)?;
                    let addrs_v6 = UniqueV6InterfaceAddressGenerator::new(count_v6).generate(d)?;
                    addrs_v4.into_iter().chain(addrs_v6).collect()
                }
                IfType::Vtep => vec![format!("{}/32", d.produce::<Ipv4AddrString>()?.0)],
            }
        } else {
            vec![]
        };

        let vlan = match r#type {
            IfType::Ethernet | IfType::Loopback | IfType::Vtep => None,
            IfType::Vlan => Some(u32::from(
                // 12 bits for VLAN ID, max of 4096 - 2
                d.gen_u16(Bound::Included(&1), Bound::Included(&(4096 - 2)))?,
            )),
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
            role: IfRole::Fabric.into(), // Dataplane only supports Fabric for now - d.produce::<IfRole>()?.into(),
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
                assert!(intf.ipaddrs.iter().all(|ifaddr| {
                    let (ip, _mask) = ifaddr.split_once('/').unwrap();
                    ip.parse::<std::net::Ipv4Addr>().is_ok()
                        || ip.parse::<std::net::Ipv6Addr>().is_ok()
                }));
                if intf.r#type == i32::from(IfType::Vtep) {
                    assert!(intf.ipaddrs.len() == 1);
                    let (ip, mask) = intf.ipaddrs[0].split_once('/').unwrap();
                    // Dataplane only supports v4 VTEP IPs right now
                    assert!(ip.parse::<std::net::Ipv4Addr>().is_ok());
                    assert_eq!(mask, "32");
                }
                assert!(intf.macaddr.is_some() || intf.r#type != i32::from(IfType::Ethernet));
                assert!(intf.vlan.is_some() || intf.r#type != i32::from(IfType::Vlan));
                assert!(intf.ospf.is_none() || intf.r#type != i32::from(IfType::Loopback));
                assert!(intf.ospf.is_none() || intf.r#type != i32::from(IfType::Vtep));
                assert!(intf.system_name.is_none());
            });
    }
}
