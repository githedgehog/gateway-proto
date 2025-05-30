// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

use crate::bolero::support::{LinuxIfName, choose};
use crate::config::{Interface, OspfConfig, RouterConfig, Vrf};
use bolero::{Driver, TypeGenerator};
use std::ops::Bound;

impl TypeGenerator for Vrf {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let router = d.produce::<RouterConfig>()?;
        let ospf = d.produce::<OspfConfig>()?;
        Some(Vrf {
            name: d.produce::<LinuxIfName>()?.0,
            interfaces: (0..d.gen_usize(Bound::Included(&1), Bound::Included(&10))?)
                .map(|_| d.produce::<Interface>())
                .collect::<Option<Vec<_>>>()?,
            router: choose(d, &[Some(router), None])?,
            ospf: choose(d, &[Some(ospf), None])?,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::config::Vrf;

    #[test]
    fn test_vrf() {
        let mut some_interfaces = false;
        let mut some_router = false;
        bolero::check!().with_type::<Vrf>().for_each(|vrf| {
            assert!(vrf.name.len() > 0);
            some_router = some_router || vrf.router.is_some();
            some_interfaces = some_interfaces || vrf.interfaces.len() > 0;
        });
        assert!(some_router);
        assert!(some_interfaces);
    }
}
