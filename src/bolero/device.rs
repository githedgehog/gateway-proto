// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::bolero::support::K8sObjectNameString;
use crate::config;
use crate::config::{Device, PacketDriver, TracingConfig};
use bolero::{Driver, TypeGenerator};
use std::collections::HashMap;
use std::ops::Bound;

impl TypeGenerator for TracingConfig {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let numtags = d.gen_u16(Bound::Included(&1), Bound::Included(&10))?;
        let mut map: HashMap<String, i32> = HashMap::new();
        let tagbase = d.produce::<String>()?;
        for k in 1..=numtags {
            let tag = format!("{tagbase}-{k}");
            let level: i32 = d.produce::<config::LogLevel>()?.into();
            map.insert(tag, level);
        }
        Some(TracingConfig {
            default: d.produce::<config::LogLevel>()?.into(),
            taglevel: map,
        })
    }
}

impl TypeGenerator for Device {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(Device {
            driver: i32::from(d.produce::<PacketDriver>()?),
            eal: None,     // TODO Add support for EAL when dataplane supports it
            ports: vec![], // TODO Add support for ports when dataplane supports it
            hostname: d.produce::<K8sObjectNameString>()?.0,
            tracing: Some(d.produce::<TracingConfig>()?),
        })
    }
}
