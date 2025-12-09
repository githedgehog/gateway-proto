// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

#![deny(clippy::all, clippy::pedantic)]

#[allow(clippy::pedantic)]
pub mod config {
    include!("generated/config.rs");
}
pub mod google {
    pub mod protobuf {
        include!("generated/google.protobuf.rs");
    }
}

// Note(manishv): This is incomplete and not needed really, remove?
// See https://github.com/githedgehog/gateway-proto/issues/28
pub use config::{
    BgpAddressFamilyIPv4,
    BgpAddressFamilyIPv6,
    BgpAddressFamilyL2vpnEvpn,
    BgpAf,
    BgpMessageCounters,
    BgpMessages,
    BgpNeighbor,
    BgpNeighborPrefixes,
    // BGP runtime
    BgpNeighborSessionState,
    BgpNeighborStatus,
    BgpNeighborUpdateSource,
    BgpStatus,

    BgpVrfStatus,
    DataplaneStatusInfo,
    DataplaneStatusType,

    // ---------- Device-related ----------
    Device,
    Eal,
    // ---------- Common / errors ----------
    Error,

    Expose,
    FrrAgentStatusType,
    FrrStatus,
    // ---------- Top-level config ----------
    GatewayConfig,
    GatewayGroup,
    GatewayGroupMember,
    GetConfigGenerationRequest,
    GetConfigGenerationResponse,
    // ---------- Requests / Responses ----------
    GetConfigRequest,
    GetDataplaneStatusRequest,
    GetDataplaneStatusResponse,

    IfRole,
    IfType,
    // ---------- Overlay ----------
    Interface,
    InterfaceAdminStatusType,
    // ---------- NEW: Extended runtime status ----------
    // Interfaces
    InterfaceCounters,
    InterfaceOperStatusType,
    InterfaceRuntimeStatus,

    // ---------- Dataplane & FRR status (existing) ----------
    InterfaceStatus,
    LogLevel,
    // ---------- Underlay ----------
    OspfConfig,
    OspfInterface,
    OspfNetworkType,
    Overlay,

    PacketDriver,

    PeeringAs,
    PeeringEntryFor,
    PeeringIPs,
    Ports,
    RouterConfig,
    Underlay,
    UpdateConfigRequest,
    UpdateConfigResponse,
    Vpc,
    VpcCounters,

    // VPC runtime
    VpcInterfaceStatus,
    VpcPeering,
    // VPC↔VPC counters
    VpcPeeringCounters,
    VpcStatus,

    Vrf,

    ZebraStatusType,
    // ---------- Service definitions ----------
    config_service_client::ConfigServiceClient,
    config_service_server::{ConfigService, ConfigServiceServer},
};

#[must_use]
pub fn get_proto_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("proto")
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "bolero")]
pub mod bolero;

mod duration;
