// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::config::{Device, GatewayConfig, Overlay, Underlay, Vpc, VpcPeering, Vrf};
use bolero::{Driver, TypeGenerator};
use std::ops::Bound;

// FIXME: Only generate peerings between vpcs named in vpc list
impl TypeGenerator for Overlay {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let nvpcs = d.gen_usize(Bound::Included(&0), Bound::Included(&10))?;
        let npeerings = d.gen_usize(Bound::Included(&0), Bound::Included(&10))?;
        let mut next_vni = 1;
        let mut peering_num = 0;
        Some(Overlay {
            vpcs: (0..nvpcs)
                .map(|_| {
                    let mut vpc = d.produce::<Vpc>()?;
                    vpc.vni = next_vni;
                    vpc.id = format!("{next_vni:05}");
                    vpc.name = format!("{next_vni:05}");
                    next_vni += 1;
                    Some(vpc)
                })
                .collect::<Option<Vec<_>>>()?,
            peerings: (0..npeerings)
                .map(|_| {
                    let mut peering = d.produce::<VpcPeering>()?;
                    peering.name = format!("peering{peering_num}");
                    peering_num += 1;
                    Some(peering)
                })
                .collect::<Option<Vec<_>>>()?,
        })
    }
}

impl TypeGenerator for Underlay {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        // Dataplane currently only supports a single vrf
        const MAX_UNDERLAY_VRFS: usize = 1;
        let nvrfs = d.gen_usize(Bound::Included(&1), Bound::Included(&MAX_UNDERLAY_VRFS))?;
        Some(Underlay {
            vrfs: (0..nvrfs)
                .map(|_| d.produce::<Vrf>())
                .collect::<Option<Vec<_>>>()?,
        })
    }
}

impl TypeGenerator for GatewayConfig {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(GatewayConfig {
            generation: d.gen_i64(Bound::Included(&0), Bound::Included(&1000))?,
            device: Some(d.produce::<Device>()?),
            overlay: Some(d.produce::<Overlay>()?),
            underlay: Some(d.produce::<Underlay>()?),
        })
    }
}
#[cfg(test)]
mod test {
    use crate::config::{Overlay, Underlay};

    #[test]
    fn test_overlay() {
        bolero::check!()
            .with_type::<Overlay>()
            .for_each(|_overlay| {
                // Other tests cover the interesting stuff, this just makes sure the generator doesn't panic
            });
    }

    #[test]
    fn test_underlay() {
        bolero::check!()
            .with_type::<Underlay>()
            .for_each(|_underlay| {
                // Other tests cover the interesting stuff, this just makes sure the generator doesn't panic
            });
    }
}
