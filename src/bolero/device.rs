// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::bolero::support::K8sObjectNameString;
use crate::config;
use crate::config::{Device, PacketDriver, TracingConfig, TracingTagConfig};
use std::ops::Bound;

use bolero::{Driver, TypeGenerator};

impl TypeGenerator for TracingTagConfig {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(TracingTagConfig {
            tag: d.produce::<String>()?,
            loglevel: d.produce::<config::LogLevel>()?.into(),
        })
    }
}

impl TypeGenerator for TracingConfig {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        let numtags = d.gen_u16(Bound::Included(&1), Bound::Included(&10))?;
        let tagconfig = (0..numtags)
            .enumerate()
            .map(|_| d.produce::<TracingTagConfig>())
            .collect::<Option<Vec<_>>>()?;

        Some(TracingConfig {
            default: d.produce::<config::LogLevel>()?.into(),
            tagconfig,
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
