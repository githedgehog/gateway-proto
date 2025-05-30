// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

#![deny(clippy::all, clippy::pedantic)]

#[allow(clippy::pedantic)]
pub mod config {
    include!("generated/config.rs");
}

// Note(manishv): This is incomplete and not needed really, remove?
// See https://github.com/githedgehog/gateway-proto/issues/28
pub use config::{
    BgpAddressFamilyIPv4,
    BgpAddressFamilyIPv6,
    BgpAddressFamilyL2vpnEvpn,
    BgpAf,
    BgpNeighbor,
    // Device related types
    Device,
    Eal,
    // Common types
    Error,

    Expose,
    GatewayConfig,
    GetConfigGenerationRequest,
    GetConfigGenerationResponse,
    // Request/Response types
    GetConfigRequest,
    IfRole,
    IfType,
    // Interface related types
    Interface,
    LogLevel,
    OspfConfig,
    OspfInterface,

    // Overlay related types
    Overlay,
    PacketDriver,

    PeeringAs,
    PeeringEntryFor,
    PeeringIPs,
    Ports,
    RouteMap,

    RouterConfig,
    // Underlay related types
    Underlay,
    UpdateConfigRequest,
    UpdateConfigResponse,

    Vpc,
    VpcPeering,
    Vrf,
    config_service_client::ConfigServiceClient,

    // Service definitions
    config_service_server::{ConfigService, ConfigServiceServer},
};

#[must_use]
pub fn get_proto_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("proto")
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "bolero")]
pub mod bolero;
