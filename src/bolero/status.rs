// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::bolero::support::{LinuxIfName, choose};
use crate::config::{
    BgpMessageCounters, BgpMessages, BgpNeighborPrefixes, BgpNeighborSessionState,
    BgpNeighborStatus, BgpStatus, BgpVrfStatus, DataplaneStatusInfo, DataplaneStatusType,
    FrrAgentStatusType, FrrStatus, GetDataplaneStatusRequest, GetDataplaneStatusResponse,
    InterfaceAdminStatusType, InterfaceCounters, InterfaceOperStatusType, InterfaceRuntimeStatus,
    InterfaceStatus, VpcCounters, VpcInterfaceStatus, VpcPeeringCounters, VpcStatus,
    ZebraStatusType,
};
use bolero::{Driver, TypeGenerator};
use std::ops::Bound;

impl TypeGenerator for InterfaceOperStatusType {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let variants = [
            InterfaceOperStatusType::InterfaceStatusUnknown,
            InterfaceOperStatusType::InterfaceStatusOperUp,
            InterfaceOperStatusType::InterfaceStatusOperDown,
            InterfaceOperStatusType::InterfaceStatusError,
        ];
        let index = d.gen_usize(Bound::Included(&0), Bound::Included(&(variants.len() - 1)))?;
        Some(variants[index])
    }
}

impl TypeGenerator for InterfaceAdminStatusType {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let variants = [
            InterfaceAdminStatusType::InterfaceAdminStatusUnknown,
            InterfaceAdminStatusType::InterfaceAdminStatusUp,
            InterfaceAdminStatusType::InterfaceAdminStatusDown,
        ];
        let index = d.gen_usize(Bound::Included(&0), Bound::Included(&(variants.len() - 1)))?;
        Some(variants[index])
    }
}

impl TypeGenerator for ZebraStatusType {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let variants = [
            ZebraStatusType::ZebraStatusNotConnected,
            ZebraStatusType::ZebraStatusConnected,
        ];
        let index = d.gen_usize(Bound::Included(&0), Bound::Included(&(variants.len() - 1)))?;
        Some(variants[index])
    }
}

impl TypeGenerator for FrrAgentStatusType {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let variants = [
            FrrAgentStatusType::FrrAgentStatusNotConnected,
            FrrAgentStatusType::FrrAgentStatusConnected,
        ];
        let index = d.gen_usize(Bound::Included(&0), Bound::Included(&(variants.len() - 1)))?;
        Some(variants[index])
    }
}

impl TypeGenerator for DataplaneStatusType {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let variants = [
            DataplaneStatusType::DataplaneStatusUnknown,
            DataplaneStatusType::DataplaneStatusHealthy,
            DataplaneStatusType::DataplaneStatusInit,
            DataplaneStatusType::DataplaneStatusError,
        ];
        let index = d.gen_usize(Bound::Included(&0), Bound::Included(&(variants.len() - 1)))?;
        Some(variants[index])
    }
}

impl TypeGenerator for GetDataplaneStatusRequest {
    fn generate<D: Driver>(_d: &mut D) -> Option<Self> {
        Some(GetDataplaneStatusRequest {})
    }
}

impl TypeGenerator for InterfaceStatus {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(InterfaceStatus {
            ifname: d.produce::<LinuxIfName>()?.0,
            oper_status: d.produce::<InterfaceOperStatusType>()?.into(),
            admin_status: d.produce::<InterfaceAdminStatusType>()?.into(),
        })
    }
}

impl TypeGenerator for FrrStatus {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let restart_weight = d.gen_u8(Bound::Included(&0), Bound::Included(&100))?;
        let restarts = match restart_weight {
            0..=80 => d.gen_u32(Bound::Included(&0), Bound::Included(&5))?,
            81..=95 => d.gen_u32(Bound::Included(&6), Bound::Included(&50))?,
            _ => d.gen_u32(Bound::Included(&51), Bound::Included(&1000))?,
        };

        Some(FrrStatus {
            zebra_status: d.produce::<ZebraStatusType>()?.into(),
            frr_agent_status: d.produce::<FrrAgentStatusType>()?.into(),
            applied_config_gen: d.produce::<i64>()?,
            restarts,
            applied_configs: d.produce::<u32>()?,
            failed_configs: d.produce::<u32>()?,
        })
    }
}

impl TypeGenerator for DataplaneStatusInfo {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(DataplaneStatusInfo {
            status: d.produce::<DataplaneStatusType>()?.into(),
        })
    }
}

#[allow(clippy::cast_precision_loss)]
impl TypeGenerator for InterfaceCounters {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let rx_bits = d.gen_u64(Bound::Included(&0), Bound::Included(&10_000_000))?;
        let tx_bits = d.gen_u64(Bound::Included(&0), Bound::Included(&10_000_000))?;
        let rx_errors = d.gen_u64(Bound::Included(&0), Bound::Included(&10_000))?;
        let tx_errors = d.gen_u64(Bound::Included(&0), Bound::Included(&10_000))?;
        let rx_bps = d.gen_u64(Bound::Included(&0), Bound::Included(&5_000_000))? as f64;
        let tx_bps = d.gen_u64(Bound::Included(&0), Bound::Included(&5_000_000))? as f64;

        Some(InterfaceCounters {
            tx_bits,
            tx_bps,
            tx_errors,
            rx_bits,
            rx_bps,
            rx_errors,
        })
    }
}

impl TypeGenerator for InterfaceRuntimeStatus {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let mtu = d.gen_u32(Bound::Included(&576), Bound::Included(&9216))?;
        let produced_ic = d.produce::<InterfaceCounters>();
        let counters_pick = choose(d, &[produced_ic, None])?;

        Some(InterfaceRuntimeStatus {
            admin_status: d.produce::<InterfaceAdminStatusType>()?.into(),
            oper_status: d.produce::<InterfaceOperStatusType>()?.into(),
            mac: format!(
                "02:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                d.gen_u8(Bound::Included(&0), Bound::Included(&255))?,
                d.gen_u8(Bound::Included(&0), Bound::Included(&255))?,
                d.gen_u8(Bound::Included(&0), Bound::Included(&255))?,
                d.gen_u8(Bound::Included(&0), Bound::Included(&255))?,
                d.gen_u8(Bound::Included(&0), Bound::Included(&255))?
            ),
            mtu,
            counters: counters_pick,
        })
    }
}

impl TypeGenerator for BgpNeighborSessionState {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let variants = [
            BgpNeighborSessionState::BgpStateUnset,
            BgpNeighborSessionState::BgpStateIdle,
            BgpNeighborSessionState::BgpStateConnect,
            BgpNeighborSessionState::BgpStateActive,
            BgpNeighborSessionState::BgpStateOpen,
            BgpNeighborSessionState::BgpStateEstablished,
        ];
        let idx = d.gen_usize(Bound::Included(&0), Bound::Included(&(variants.len() - 1)))?;
        Some(variants[idx])
    }
}

impl TypeGenerator for BgpMessageCounters {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(BgpMessageCounters {
            capability: d.gen_u64(Bound::Included(&0), Bound::Included(&10_000))?,
            keepalive: d.gen_u64(Bound::Included(&0), Bound::Included(&10_000))?,
            notification: d.gen_u64(Bound::Included(&0), Bound::Included(&1_000))?,
            open: d.gen_u64(Bound::Included(&0), Bound::Included(&5_000))?,
            route_refresh: d.gen_u64(Bound::Included(&0), Bound::Included(&5_000))?,
            update: d.gen_u64(Bound::Included(&0), Bound::Included(&50_000))?,
        })
    }
}

impl TypeGenerator for BgpMessages {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(BgpMessages {
            received: Some(d.produce::<BgpMessageCounters>()?),
            sent: Some(d.produce::<BgpMessageCounters>()?),
        })
    }
}

impl TypeGenerator for BgpNeighborPrefixes {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let received_pre = d.gen_u32(Bound::Included(&0), Bound::Included(&50_000))?;
        let received = d.gen_u32(Bound::Included(&0), Bound::Included(&received_pre))?;
        let sent = d.gen_u32(Bound::Included(&0), Bound::Included(&50_000))?;
        Some(BgpNeighborPrefixes {
            received,
            received_pre_policy: received_pre,
            sent,
        })
    }
}

impl TypeGenerator for BgpNeighborStatus {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let peer_port = d.gen_u32(Bound::Included(&1), Bound::Included(&65535))?;
        let local_as = d.gen_u32(Bound::Included(&1), Bound::Included(&65_534))?;
        let peer_as = d.gen_u32(Bound::Included(&1), Bound::Included(&65_534))?;

        Some(BgpNeighborStatus {
            enabled: d.gen_bool(None)?,
            local_as,
            peer_as,
            peer_port,
            peer_group: format!(
                "grp{}",
                d.gen_u32(Bound::Included(&0), Bound::Included(&1000))?
            ),
            remote_router_id: format!(
                "{}.{}.{}.{}",
                d.gen_u8(Bound::Included(&1), Bound::Included(&254))?,
                d.gen_u8(Bound::Included(&0), Bound::Included(&255))?,
                d.gen_u8(Bound::Included(&0), Bound::Included(&255))?,
                d.gen_u8(Bound::Included(&1), Bound::Included(&254))?
            ),
            session_state: d.produce::<BgpNeighborSessionState>()?.into(),
            connections_dropped: d.gen_u64(Bound::Included(&0), Bound::Included(&1000))?,
            established_transitions: d.gen_u64(Bound::Included(&0), Bound::Included(&1000))?,
            last_reset_reason: "test".into(),
            messages: Some(d.produce::<BgpMessages>()?),
            ipv4_unicast_prefixes: Some(d.produce::<BgpNeighborPrefixes>()?),
            ipv6_unicast_prefixes: Some(d.produce::<BgpNeighborPrefixes>()?),
            l2vpn_evpn_prefixes: Some(d.produce::<BgpNeighborPrefixes>()?),
        })
    }
}

impl TypeGenerator for BgpVrfStatus {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let n = d.gen_usize(Bound::Included(&0), Bound::Included(&4))?;
        let mut neighbors = std::collections::HashMap::new();
        for _ in 0..n {
            let ip = format!(
                "10.{}.{}.{}",
                d.gen_u8(Bound::Included(&0), Bound::Included(&255))?,
                d.gen_u8(Bound::Included(&0), Bound::Included(&255))?,
                d.gen_u8(Bound::Included(&1), Bound::Included(&254))?
            );
            neighbors
                .entry(ip)
                .or_insert_with(|| d.produce::<BgpNeighborStatus>().unwrap());
        }
        Some(BgpVrfStatus { neighbors })
    }
}

impl TypeGenerator for BgpStatus {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let nvrfs = d.gen_usize(Bound::Included(&0), Bound::Included(&3))?;
        let mut vrfs = std::collections::HashMap::new();
        for i in 0..nvrfs {
            let name = if i == 0 {
                "default".into()
            } else {
                format!("vrf{i}")
            };
            vrfs.insert(name, d.produce::<BgpVrfStatus>()?);
        }
        Some(BgpStatus { vrfs })
    }
}

impl TypeGenerator for VpcInterfaceStatus {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(VpcInterfaceStatus {
            ifname: d.produce::<LinuxIfName>()?.0,
            admin_status: d.produce::<InterfaceAdminStatusType>()?.into(),
            oper_status: d.produce::<InterfaceOperStatusType>()?.into(),
        })
    }
}

impl TypeGenerator for VpcStatus {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let name = format!(
            "vpc-{}",
            d.gen_u32(Bound::Included(&1), Bound::Included(&128))?
        );
        let id = format!(
            "id-{}",
            d.gen_u32(Bound::Included(&1), Bound::Included(&10_000))?
        );
        let vni = d.gen_u32(Bound::Included(&1), Bound::Included(&16_777_215))?;
        let route_count = d.gen_u32(Bound::Included(&0), Bound::Included(&50_000))?;

        let nifs = d.gen_usize(Bound::Included(&0), Bound::Included(&4))?;
        let mut interfaces = std::collections::HashMap::new();
        for _ in 0..nifs {
            let s = d.produce::<VpcInterfaceStatus>()?;
            interfaces.insert(s.ifname.clone(), s);
        }

        Some(VpcStatus {
            id,
            name,
            vni,
            route_count,
            interfaces,
        })
    }
}

#[allow(clippy::cast_precision_loss)]
impl TypeGenerator for VpcPeeringCounters {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let a = format!(
            "vpc-{}",
            d.gen_u32(Bound::Included(&1), Bound::Included(&64))?
        );
        let b = format!(
            "vpc-{}",
            d.gen_u32(Bound::Included(&1), Bound::Included(&64))?
        );
        let (src_vpc, dst_vpc) = if a <= b { (a, b) } else { (b, a) };
        let packets = d.gen_u64(Bound::Included(&0), Bound::Included(&10_000_000))?;
        let bytes = packets.saturating_mul(64);
        let drops = d.gen_u64(Bound::Included(&0), Bound::Included(&packets))?;
        let pps = d.gen_u64(Bound::Included(&0), Bound::Included(&100_000))? as f64;
        let bps = d.gen_u64(Bound::Included(&0), Bound::Included(&5_000_000))? as f64;

        Some(VpcPeeringCounters {
            name: format!("{src_vpc}--{dst_vpc}"),
            src_vpc,
            dst_vpc,
            packets,
            bytes,
            drops,
            pps,
            bps,
        })
    }
}

impl TypeGenerator for VpcCounters {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let name = format!(
            "vpc-{}",
            d.gen_u32(Bound::Included(&1), Bound::Included(&128))?
        );
        let packets = d.gen_u64(Bound::Included(&0), Bound::Included(&100_000_000))?;
        let drops = d.gen_u64(Bound::Included(&0), Bound::Included(&packets))?;
        let bytes = packets.saturating_mul(64);
        Some(VpcCounters {
            name,
            packets,
            drops,
            bytes,
        })
    }
}

impl TypeGenerator for GetDataplaneStatusResponse {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        // 0..=8 interface statuses, unique names
        let ninterfaces = d.gen_usize(Bound::Included(&0), Bound::Included(&8))?;
        let mut interface_statuses = Vec::new();
        let mut used_names = std::collections::HashSet::new();

        for _ in 0..ninterfaces {
            let mut attempts = 0;
            while attempts < 20 {
                if let Some(interface) = d.produce::<InterfaceStatus>() {
                    if used_names.insert(interface.ifname.clone()) {
                        interface_statuses.push(interface);
                        break;
                    }
                }
                attempts += 1;
            }
        }

        let frr_produced_status = d.produce::<FrrStatus>()?;
        let frr_status = choose(d, &[Some(frr_produced_status), None])?;
        let produced_status = d.produce::<DataplaneStatusInfo>()?;
        let dataplane_status = choose(d, &[Some(produced_status), None])?;

        // Interface runtime map: subset of names + possibly extra ones
        let mut interface_runtime = std::collections::HashMap::new();
        let target = d.gen_usize(Bound::Included(&0), Bound::Included(&(ninterfaces + 2)))?;

        let mut pool: Vec<String> = used_names.iter().cloned().collect();
        let extra = d.gen_usize(Bound::Included(&0), Bound::Included(&2))?;
        for _ in 0..extra {
            pool.push(format!(
                "if{}",
                d.gen_u32(Bound::Included(&0), Bound::Included(&9999))?
            ));
        }

        for name in pool.into_iter().take(target) {
            interface_runtime.insert(name, d.produce::<InterfaceRuntimeStatus>()?);
        }

        let bgp_produced = d.produce::<BgpStatus>()?;
        let bgp = choose(d, &[Some(bgp_produced), None])?;

        // VPCs map (0..=4)
        let nvpcs = d.gen_usize(Bound::Included(&0), Bound::Included(&4))?;
        let mut vpcs = std::collections::HashMap::new();
        for _ in 0..nvpcs {
            let v = d.produce::<VpcStatus>()?;
            vpcs.insert(v.name.clone(), v);
        }

        // VPC peering counters (0..=6)
        let npeers = d.gen_usize(Bound::Included(&0), Bound::Included(&6))?;
        let mut vpc_peering_counters = std::collections::HashMap::new();
        for _ in 0..npeers {
            let c = d.produce::<VpcPeeringCounters>()?;
            vpc_peering_counters.insert(c.name.clone(), c);
        }

        let nvc = d.gen_usize(Bound::Included(&0), Bound::Included(&4))?;
        let mut vpc_counters = std::collections::HashMap::new();
        for _ in 0..nvc {
            let c = d.produce::<VpcCounters>()?;
            vpc_counters.insert(c.name.clone(), c);
        }

        Some(GetDataplaneStatusResponse {
            interface_statuses,
            frr_status,
            dataplane_status,
            interface_runtime,
            bgp,
            vpcs,
            vpc_peering_counters,
            vpc_counters,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_interface_status_type() {
        bolero::check!()
            .with_type::<InterfaceOperStatusType>()
            .for_each(|t| {
                let n = i32::from(*t);
                assert!((0..=3).contains(&n));
            });
    }

    #[test]
    fn test_frr_status_type() {
        bolero::check!()
            .with_type::<ZebraStatusType>()
            .for_each(|t| {
                let n = i32::from(*t);
                assert!((0..=1).contains(&n));
            });
    }

    #[test]
    fn test_frr_agent_status_type() {
        bolero::check!()
            .with_type::<FrrAgentStatusType>()
            .for_each(|t| {
                let n = i32::from(*t);
                assert!((0..=1).contains(&n));
            });
    }

    #[test]
    fn test_interface_admin_status_type() {
        bolero::check!()
            .with_type::<InterfaceAdminStatusType>()
            .for_each(|t| {
                let n = i32::from(*t);
                assert!((0..=2).contains(&n));
            });
    }

    #[test]
    fn test_dataplane_status_type() {
        bolero::check!()
            .with_type::<DataplaneStatusType>()
            .for_each(|t| {
                let n = i32::from(*t);
                assert!((0..=3).contains(&n));
            });
    }

    #[test]
    fn test_interface_status() {
        bolero::check!()
            .with_type::<InterfaceStatus>()
            .for_each(|s| {
                assert!(!s.ifname.is_empty());
                assert!((0..=3).contains(&s.oper_status));
                assert!((0..=2).contains(&s.admin_status));
            });
    }

    #[test]
    fn test_frr_status() {
        let mut some_restarts = false;
        bolero::check!().with_type::<FrrStatus>().for_each(|frr| {
            assert!((0..=1).contains(&frr.zebra_status));
            assert!((0..=1).contains(&frr.frr_agent_status));
            assert!(frr.restarts <= 1000);
            if frr.restarts > 0 {
                some_restarts = true;
            }
        });
        assert!(some_restarts);
    }

    #[test]
    fn test_dataplane_status_info() {
        bolero::check!()
            .with_type::<DataplaneStatusInfo>()
            .for_each(|dsi| {
                assert!((0..=3).contains(&dsi.status));
            });
    }

    #[test]
    fn test_get_dataplane_status_request() {
        bolero::check!()
            .with_type::<GetDataplaneStatusRequest>()
            .for_each(|_request| {
                // Empty request: nothing to validate
            });
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_get_dataplane_status_response() {
        let mut some_interfaces = false;
        let mut some_frr_status = false;
        let mut some_dataplane_status = false;
        let mut missing_frr_status = false;
        let mut missing_dataplane_status = false;

        bolero::check!()
            .with_type::<GetDataplaneStatusResponse>()
            .for_each(|resp| {
                assert!(resp.interface_statuses.len() <= 8);

                // unique names
                let mut seen = std::collections::HashSet::new();
                for iface in &resp.interface_statuses {
                    assert!(seen.insert(&iface.ifname), "dup ifname {}", iface.ifname);
                    assert!(!iface.ifname.is_empty());
                    assert!((0..=3).contains(&iface.oper_status));
                    assert!((0..=2).contains(&iface.admin_status));
                }
                if !resp.interface_statuses.is_empty() {
                    some_interfaces = true;
                }

                // FRR optional
                if let Some(frr) = &resp.frr_status {
                    some_frr_status = true;
                    assert!((0..=1).contains(&frr.zebra_status));
                    assert!((0..=1).contains(&frr.frr_agent_status));
                    assert!(frr.restarts <= 1000);
                } else {
                    missing_frr_status = true;
                }

                // Dataplane optional
                if let Some(dp) = &resp.dataplane_status {
                    some_dataplane_status = true;
                    assert!((0..=3).contains(&dp.status));
                } else {
                    missing_dataplane_status = true;
                }

                // interface_runtime values must be sane if present
                for (k, v) in &resp.interface_runtime {
                    assert!(!k.is_empty());
                    assert!((0..=3).contains(&v.oper_status));
                    assert!((0..=2).contains(&v.admin_status));
                }

                // bgp (optional)
                if let Some(bgp) = &resp.bgp {
                    for (vrf, vrf_status) in &bgp.vrfs {
                        assert!(!vrf.is_empty());
                        for (nbr, st) in &vrf_status.neighbors {
                            assert!(!nbr.is_empty());
                            assert!(st.peer_port <= 65535);
                            assert!((0..=5).contains(&st.session_state));
                            if let Some(msgs) = &st.messages {
                                if let Some(r) = &msgs.received {
                                    let _ = (
                                        r.capability,
                                        r.keepalive,
                                        r.notification,
                                        r.open,
                                        r.route_refresh,
                                        r.update,
                                    );
                                }
                                if let Some(s) = &msgs.sent {
                                    let _ = (
                                        s.capability,
                                        s.keepalive,
                                        s.notification,
                                        s.open,
                                        s.route_refresh,
                                        s.update,
                                    );
                                }
                            }
                            // check per-AF vectors
                            let Some(p) = &st.ipv4_unicast_prefixes else {
                                todo!()
                            };
                            assert!(p.received <= p.received_pre_policy);
                            let Some(p) = &st.ipv6_unicast_prefixes else {
                                todo!()
                            };
                            assert!(p.received <= p.received_pre_policy);
                            let Some(p) = &st.l2vpn_evpn_prefixes else {
                                todo!()
                            };
                            assert!(p.received <= p.received_pre_policy);
                        }
                    }
                }

                // VPC status map (keys are VPC names)
                for (k, v) in &resp.vpcs {
                    assert_eq!(k, &v.name);
                    assert!(!v.id.is_empty());
                    assert!((1..=16_777_215).contains(&v.vni));
                }

                // Peering counters map
                for (name, c) in &resp.vpc_peering_counters {
                    assert_eq!(name, &c.name);
                    assert!(!c.src_vpc.is_empty());
                    assert!(!c.dst_vpc.is_empty());
                    assert!(c.packets >= c.drops);
                    assert!(c.pps >= 0.0);
                    assert!(c.bps >= 0.0);
                }

                for (name, c) in &resp.vpc_counters {
                    assert_eq!(name, &c.name);
                    assert!(!c.name.is_empty());
                    assert!(c.packets >= c.drops);
                }
            });

        assert!(some_interfaces);
        assert!(some_frr_status);
        assert!(some_dataplane_status);
        assert!(missing_frr_status);
        assert!(missing_dataplane_status);
    }

    #[test]
    fn test_weighted_restart_generation() {
        let mut low_restarts = 0;
        let mut medium_restarts = 0;
        let mut high_restarts = 0;
        let total_samples = 1000;

        bolero::check!()
            .with_type::<FrrStatus>()
            .with_iterations(total_samples)
            .for_each(|frr_status| match frr_status.restarts {
                0..=5 => low_restarts += 1,
                6..=50 => medium_restarts += 1,
                51..=1000 => high_restarts += 1,
                _ => panic!(
                    "Restart count out of expected range: {}",
                    frr_status.restarts
                ),
            });

        assert!(low_restarts + medium_restarts + high_restarts > 0);
    }
}
