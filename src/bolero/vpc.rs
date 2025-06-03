// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::bolero::support::{ALPHA_NUMERIC_CHARS, LinuxIfName, gen_from_chars};
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
        Some(VpcPeering {
            name: d.produce::<LinuxIfName>()?.0,
            r#for: (0..2)
                .map(|_| d.produce::<PeeringEntryFor>())
                .collect::<Option<Vec<_>>>()?,
        })
    }
}

impl TypeGenerator for Vpc {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let nintf = d.gen_usize(Bound::Included(&1), Bound::Included(&10))?;
        let mut iface_num = 0;
        Some(Vpc {
            name: d.produce::<LinuxIfName>()?.0,
            id: gen_from_chars(
                d,
                ALPHA_NUMERIC_CHARS,
                Bound::Included(&5),
                Bound::Included(&5),
            )?,
            vni: d.gen_u32(Bound::Included(&1), Bound::Excluded(&(1 << 20)))?,
            interfaces: (0..nintf)
                .map(|_| {
                    let mut iface = d.produce::<Interface>()?;
                    let mut name = d.produce::<LinuxIfName>()?.0;
                    if !name.is_empty() {
                        name.pop();
                    }
                    iface.name = format!("{name}{iface_num}");
                    iface_num += 1;
                    Some(iface)
                })
                .collect::<Option<Vec<_>>>()?,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::config::{PeeringEntryFor, VpcPeering};

    #[test]
    fn test_peering_entry_for() {
        bolero::check!()
            .with_type::<PeeringEntryFor>()
            .for_each(|peering_entry_for| {
                assert!(!peering_entry_for.vpc.is_empty());
                assert!(!peering_entry_for.expose.is_empty());
            });
    }

    #[test]
    fn test_vpc_peering() {
        bolero::check!()
            .with_type::<VpcPeering>()
            .for_each(|vpc_peering| {
                assert!(!vpc_peering.name.is_empty());
                assert!(!vpc_peering.r#for.is_empty());
            });
    }
}
