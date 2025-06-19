// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::bolero::support::{
    CidrString, IpAddrString, Ipv4AddrString, K8sObjectNameString, LinuxIfName,
    UniqueV4CidrGenerator, UniqueV6CidrGenerator, choose,
};
use crate::config::{
    BgpAddressFamilyIPv4, BgpAddressFamilyIPv6, BgpAddressFamilyL2vpnEvpn, BgpAf, BgpNeighbor,
    BgpNeighborUpdateSource, RouteMap, RouterConfig, bgp_neighbor_update_source,
};
use bolero::{Driver, TypeGenerator, ValueGenerator};
use std::ops::Bound;

#[derive(Clone, Copy, PartialEq, TypeGenerator)]
enum BgpNeighborUpdateSourceType {
    Address,
    Interface,
}

impl TypeGenerator for BgpAddressFamilyIPv4 {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let v4_prefix_generator = UniqueV4CidrGenerator::new(
            d.gen_u16(Bound::Included(&0), Bound::Included(&10))?,
            d.gen_u8(Bound::Included(&1), Bound::Included(&32))?,
        );
        Some(BgpAddressFamilyIPv4 {
            redistribute_connected: d.gen_bool(None)?,
            redistribute_static: d.gen_bool(None)?,
            networks: v4_prefix_generator.generate(d)?,
        })
    }
}

impl TypeGenerator for BgpAddressFamilyIPv6 {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let v6_prefix_generator = UniqueV6CidrGenerator::new(
            d.gen_u16(Bound::Included(&0), Bound::Included(&10))?,
            d.gen_u8(Bound::Included(&1), Bound::Included(&128))?,
        );
        Some(BgpAddressFamilyIPv6 {
            redistribute_connected: d.gen_bool(None)?,
            redistribute_static: d.gen_bool(None)?,
            networks: v6_prefix_generator.generate(d)?,
        })
    }
}

impl TypeGenerator for bgp_neighbor_update_source::Source {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let source: BgpNeighborUpdateSourceType = d.produce()?;

        match source {
            BgpNeighborUpdateSourceType::Address => Some(
                bgp_neighbor_update_source::Source::Address(d.produce::<IpAddrString>()?.0),
            ),
            BgpNeighborUpdateSourceType::Interface => Some(
                bgp_neighbor_update_source::Source::Interface(d.produce::<LinuxIfName>()?.0),
            ),
        }
    }
}

impl TypeGenerator for BgpNeighborUpdateSource {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(BgpNeighborUpdateSource {
            source: Some(d.produce::<bgp_neighbor_update_source::Source>())?,
        })
    }
}

impl TypeGenerator for BgpNeighbor {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let naf = d.gen_usize(Bound::Included(&0), Bound::Included(&2))?;
        let af_activate_set: std::collections::HashSet<_> = (0..naf)
            .map(|_| d.produce::<BgpAf>())
            .collect::<Option<std::collections::HashSet<_>>>()?;
        Some(BgpNeighbor {
            address: d.produce::<IpAddrString>()?.0,
            remote_asn: d.produce::<u32>()?.to_string(),
            #[allow(clippy::redundant_closure_for_method_calls)]
            af_activate: af_activate_set.into_iter().map(|af| af.into()).collect(),
            update_source: Some(d.produce::<BgpNeighborUpdateSource>()?),
        })
    }
}

// TODO: Implement this properly when dataplane supports route maps
impl TypeGenerator for RouteMap {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let nprefixes = d.gen_usize(Bound::Included(&0), Bound::Included(&10))?;
        let match_prefix_lists = (0..nprefixes)
            .map(|_| d.produce::<CidrString>().map(|prefix| prefix.0))
            .collect::<Option<Vec<_>>>()?;
        Some(RouteMap {
            name: d.produce::<K8sObjectNameString>()?.0,
            match_prefix_lists,
            action: d.produce::<String>()?,
            sequence: d.produce::<u32>()?,
        })
    }
}

impl TypeGenerator for RouterConfig {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let nneighbors = d.gen_usize(Bound::Included(&0), Bound::Included(&10))?;
        let ipv4_family = d.produce::<BgpAddressFamilyIPv4>()?;
        let ipv6_family = d.produce::<BgpAddressFamilyIPv6>()?;
        let l2vpn_evpn = d.produce::<BgpAddressFamilyL2vpnEvpn>()?;
        let ipv4_unicast = choose(d, &[Some(ipv4_family), None])?;
        let ipv6_unicast = choose(d, &[Some(ipv6_family), None])?;
        let l2vpn_evpn = choose(d, &[Some(l2vpn_evpn), None])?;
        Some(RouterConfig {
            asn: d.produce::<u32>()?.to_string(),
            router_id: d.produce::<Ipv4AddrString>()?.0, // TODO: Add ipv6 support when dataplane supports it
            neighbors: (0..nneighbors)
                .map(|_| d.produce::<BgpNeighbor>())
                .collect::<Option<Vec<_>>>()?,
            ipv4_unicast,
            ipv6_unicast,
            l2vpn_evpn,
            route_maps: vec![], // TODO: Add route maps when dataplane supports it
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bgp_neighbor_update_source() {
        bolero::check!()
            .with_type::<BgpNeighborUpdateSource>()
            .for_each(|bgp_neighbor_update_source| {
                assert!(bgp_neighbor_update_source.source.is_some());
            });
    }

    #[test]
    fn test_bgp_neighbor() {
        let mut some_afs = false;
        bolero::check!()
            .with_type::<BgpNeighbor>()
            .for_each(|bgp_neighbor| {
                assert!(bgp_neighbor.remote_asn.parse::<u32>().is_ok());
                if !bgp_neighbor.af_activate.is_empty() {
                    some_afs = true;
                }
            });
        assert!(some_afs);
    }

    #[test]
    fn test_router_config() {
        bolero::check!()
            .with_type::<RouterConfig>()
            .for_each(|router_config| {
                assert!(router_config.asn.parse::<u32>().is_ok());
            });
    }

    #[test]
    fn test_bgp_address_family_ipv4() {
        let mut some_networks = false;
        bolero::check!()
            .with_type::<BgpAddressFamilyIPv4>()
            .for_each(|bgp_address_family_ipv4| {
                if !bgp_address_family_ipv4.networks.is_empty() {
                    some_networks = true;
                    let mut seen = std::collections::HashSet::new();
                    for network in &bgp_address_family_ipv4.networks {
                        assert!(seen.insert(network), "Duplicate network found: {network}");
                    }
                }
            });
        assert!(some_networks);
    }

    #[test]
    fn test_bgp_address_family_ipv6() {
        let mut some_networks = false;
        bolero::check!()
            .with_type::<BgpAddressFamilyIPv6>()
            .for_each(|bgp_address_family_ipv6| {
                if !bgp_address_family_ipv6.networks.is_empty() {
                    some_networks = true;
                    let mut seen = std::collections::HashSet::new();
                    for network in &bgp_address_family_ipv6.networks {
                        assert!(seen.insert(network), "Duplicate network found: {network}");
                    }
                }
            });
        assert!(some_networks);
    }
}
