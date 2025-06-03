// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

use crate::bolero::support::K8sObjectNameString;
use crate::config::{Device, LogLevel, PacketDriver};
use bolero::{Driver, TypeGenerator};

impl TypeGenerator for Device {
    fn generate<D: Driver>(d: &mut D) -> Option<Self> {
        Some(Device {
            driver: i32::from(d.produce::<PacketDriver>()?),
            eal: None,     // TODO Add support for EAL when dataplane supports it
            ports: vec![], // TODO Add support for ports when dataplane supports it
            hostname: d.produce::<K8sObjectNameString>()?.0,
            loglevel: i32::from(d.produce::<LogLevel>()?),
        })
    }
}
