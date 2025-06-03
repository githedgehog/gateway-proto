// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::bolero::support::{LinuxIfName, LinuxIfNamesGenerator, choose};
use crate::config::{Interface, OspfConfig, RouterConfig, Vrf};
use bolero::{Driver, TypeGenerator, ValueGenerator};
use std::ops::Bound;

impl TypeGenerator for Vrf {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let router = d.produce::<RouterConfig>()?;
        let ospf = d.produce::<OspfConfig>()?;
        let ninterfaces = d.gen_u16(Bound::Included(&1), Bound::Included(&10))?;
        let gen_if_names = LinuxIfNamesGenerator { count: ninterfaces };
        let if_names = gen_if_names.generate(d)?;
        let interfaces = (0..ninterfaces)
            .enumerate()
            .map(|(i, _)| {
                let mut intf = d.produce::<Interface>()?;
                intf.name.clone_from(&if_names[i]);
                Some(intf)
            })
            .collect::<Option<Vec<_>>>()?;

        Some(Vrf {
            name: d.produce::<LinuxIfName>()?.0,
            interfaces,
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
            assert!(!vrf.name.is_empty());
            some_router = some_router || vrf.router.is_some();
            some_interfaces = some_interfaces || !vrf.interfaces.is_empty();
        });
        assert!(some_router);
        assert!(some_interfaces);
    }
}
