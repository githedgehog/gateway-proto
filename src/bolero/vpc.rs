// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

use crate::bolero::support::LinuxIfName;
use crate::config::{Expose, Interface, PeeringEntryFor, Vpc, VpcPeering};
use bolero::{Driver, TypeGenerator};
use std::ops::Bound;
impl TypeGenerator for PeeringEntryFor {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(PeeringEntryFor {
            vpc: d.produce::<LinuxIfName>()?.0,
            expose: vec![d.produce::<Expose>()?],
        })
    }
}

impl TypeGenerator for VpcPeering {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let npeerings = d.gen_usize(Bound::Included(&1), Bound::Included(&10))?;
        Some(VpcPeering {
            name: d.produce::<LinuxIfName>()?.0,
            r#for: (0..npeerings)
                .map(|_| d.produce::<PeeringEntryFor>())
                .collect::<Option<Vec<_>>>()?,
        })
    }
}

impl TypeGenerator for Vpc {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let nintf = d.gen_usize(Bound::Included(&1), Bound::Included(&10))?;
        Some(Vpc {
            name: d.produce::<LinuxIfName>()?.0,
            id: d.produce::<LinuxIfName>()?.0,
            vni: d.gen_u32(Bound::Included(&1), Bound::Excluded(&(1 << 20)))?,
            interfaces: (0..nintf)
                .map(|_| d.produce::<Interface>())
                .collect::<Option<Vec<_>>>()?,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::config::{Overlay, PeeringEntryFor, VpcPeering};

    #[test]
    fn test_peering_entry_for() {
        bolero::check!()
            .with_type::<PeeringEntryFor>()
            .for_each(|peering_entry_for| {
                assert!(peering_entry_for.vpc.len() > 0);
                assert!(peering_entry_for.expose.len() > 0);
            });
    }

    #[test]
    fn test_vpc_peering() {
        bolero::check!()
            .with_type::<VpcPeering>()
            .for_each(|vpc_peering| {
                assert!(vpc_peering.name.len() > 0);
                assert!(vpc_peering.r#for.len() > 0);
            });
    }
}
