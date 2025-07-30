// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::bolero::support::{LinuxIfName, choose};
use crate::config::{
    DataplaneStatusInfo, DataplaneStatusType, FrrAgentStatusType, FrrStatus,
    GetDataplaneStatusRequest, GetDataplaneStatusResponse, InterfaceAdminStatusType,
    InterfaceStatus, InterfaceStatusType, ZebraStatusType,
};
use bolero::{Driver, TypeGenerator};
use std::ops::Bound;

impl TypeGenerator for InterfaceStatusType {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let variants = [
            InterfaceStatusType::InterfaceStatusUnknown,
            InterfaceStatusType::InterfaceStatusOperUp,
            InterfaceStatusType::InterfaceStatusOperDown,
            InterfaceStatusType::InterfaceStatusError,
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
        // Empty request message
        Some(GetDataplaneStatusRequest {})
    }
}

impl TypeGenerator for InterfaceStatus {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(InterfaceStatus {
            ifname: d.produce::<LinuxIfName>()?.0,
            status: d.produce::<InterfaceStatusType>()?.into(),
            admin_status: d.produce::<InterfaceAdminStatusType>()?.into(),
        })
    }
}

impl TypeGenerator for FrrStatus {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        // Generate a weighted distribution for restarts, some random logic to simulate realistic scenarios
        // More close to Gaussian distribution
        let restart_weight = d.gen_u8(Bound::Included(&0), Bound::Included(&100))?;
        let restarts = match restart_weight {
            0..=80 => d.gen_u32(Bound::Included(&0), Bound::Included(&5))?,
            81..=95 => d.gen_u32(Bound::Included(&6), Bound::Included(&50))?,
            _ => d.gen_u32(Bound::Included(&51), Bound::Included(&1000))?,
        };

        Some(FrrStatus {
            zebra_status: d.produce::<ZebraStatusType>()?.into(),
            frr_agent_status: d.produce::<FrrAgentStatusType>()?.into(),
            applied_config_gen: d.produce::<u32>()?,
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

impl TypeGenerator for GetDataplaneStatusResponse {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        // Generate 0-8 interfaces, probably
        let ninterfaces = d.gen_usize(Bound::Included(&0), Bound::Included(&8))?;

        // Generate statuses
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

        let frr_status = d.produce::<FrrStatus>()?;
        let dataplane_status = d.produce::<DataplaneStatusInfo>()?;

        // Use choose helper to sometimes make optional fields None for testing
        let frr_status = choose(d, &[Some(frr_status), None])?;
        let dataplane_status = choose(d, &[Some(dataplane_status), None])?;

        Some(GetDataplaneStatusResponse {
            interface_statuses,
            frr_status,
            dataplane_status,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_interface_status_type() {
        bolero::check!()
            .with_type::<InterfaceStatusType>()
            .for_each(|interface_status_type| {
                let status_num = i32::from(*interface_status_type);
                assert!((0..=4).contains(&status_num));
            });
    }

    #[test]
    fn test_frr_status_type() {
        bolero::check!()
            .with_type::<ZebraStatusType>()
            .for_each(|zebra_status_type| {
                let status_num = i32::from(*zebra_status_type);
                assert!((0..=1).contains(&status_num));
            });
    }

    #[test]
    fn test_frr_agent_status_type() {
        bolero::check!()
            .with_type::<FrrAgentStatusType>()
            .for_each(|frr_agent_status_type| {
                let status_num = i32::from(*frr_agent_status_type);
                assert!((0..=1).contains(&status_num));
            });
    }

    #[test]
    fn test_interface_admin_status_type() {
        bolero::check!()
            .with_type::<InterfaceAdminStatusType>()
            .for_each(|interface_admin_status_type| {
                let status_num = i32::from(*interface_admin_status_type);
                assert!((0..=3).contains(&status_num));
            });
    }

    #[test]
    fn test_dataplane_status_type() {
        bolero::check!()
            .with_type::<DataplaneStatusType>()
            .for_each(|dataplane_status_type| {
                let status_num = i32::from(*dataplane_status_type);
                assert!((0..=3).contains(&status_num));
            });
    }

    #[test]
    fn test_interface_status() {
        bolero::check!()
            .with_type::<InterfaceStatus>()
            .for_each(|interface_status| {
                assert!(!interface_status.ifname.is_empty());
                assert!((0..=4).contains(&interface_status.status));
            });
    }

    #[test]
    fn test_frr_status() {
        let mut some_restarts = false;
        bolero::check!()
            .with_type::<FrrStatus>()
            .for_each(|frr_status| {
                assert!((0..=1).contains(&frr_status.zebra_status));
                // The upper limit of 1000 for frr_status.restarts is chosen as a conservative
                // estimate to ensure reasonable behavior. This value is based on expected
                // operational constraints and typical restart counts observed in similar systems.
                assert!(frr_status.restarts <= 1000);
                if frr_status.restarts > 0 {
                    some_restarts = true;
                }
                assert!((0..=1).contains(&frr_status.frr_agent_status));
                assert!(frr_status.applied_config_gen > 0);
            });
        assert!(some_restarts);
    }

    #[test]
    fn test_dataplane_status_info() {
        bolero::check!()
            .with_type::<DataplaneStatusInfo>()
            .for_each(|dataplane_status_info| {
                assert!((0..=3).contains(&dataplane_status_info.status));
            });
    }

    #[test]
    fn test_get_dataplane_status_request() {
        bolero::check!()
            .with_type::<GetDataplaneStatusRequest>()
            .for_each(|_request| {
                // Empty request should always succeed
                // No fields to validate
            });
    }

    #[test]
    fn test_get_dataplane_status_response() {
        let mut some_interfaces = false;
        let mut some_frr_status = false;
        let mut some_dataplane_status = false;
        let mut missing_frr_status = false;
        let mut missing_dataplane_status = false;

        bolero::check!()
            .with_type::<GetDataplaneStatusResponse>()
            .for_each(|response| {
                assert!(response.interface_statuses.len() <= 8);

                // Check for unique interface names
                let mut seen_names = std::collections::HashSet::new();
                for interface in &response.interface_statuses {
                    assert!(
                        seen_names.insert(&interface.ifname),
                        "Duplicate interface name found: {}",
                        interface.ifname
                    );
                    assert!(!interface.ifname.is_empty());
                    assert!((0..=4).contains(&interface.status));
                }

                if !response.interface_statuses.is_empty() {
                    some_interfaces = true;
                }

                if response.frr_status.is_some() {
                    some_frr_status = true;
                    let frr = response.frr_status.as_ref().unwrap();
                    assert!((0..=1).contains(&frr.zebra_status));
                    assert!((0..=1).contains(&frr.frr_agent_status));
                    assert!(frr.restarts <= 1000);
                } else {
                    missing_frr_status = true;
                }

                if response.dataplane_status.is_some() {
                    some_dataplane_status = true;
                    let dataplane = response.dataplane_status.as_ref().unwrap();
                    assert!((0..=3).contains(&dataplane.status));
                } else {
                    missing_dataplane_status = true;
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
    }
}
