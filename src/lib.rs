// Copyright 2025 Hedgehog
// SPDX-License-Identifier: Apache-2.0

pub mod config {
    include!("generated/config.rs");
}

pub use config::{
    // Service definitions
    config_service_server::{ConfigService, ConfigServiceServer},
    config_service_client::ConfigServiceClient,
    
    // Request/Response types
    GetConfigRequest,
    GatewayConfig,
    GetConfigGenerationRequest,
    GetConfigGenerationResponse,
    UpdateConfigRequest,
    UpdateConfigResponse,
    
    // Common types
    Error,
    
    // Device related types
    Device, Ports, Eal, LogLevel, PacketDriver,
    
    // Interface related types
    Interface, IfType, IfRole, OspfConfig, OspfInterface,
    
    // Underlay related types
    Underlay, Vrf, RouterConfig, BgpNeighbor, BgpAf, 
    BgpAddressFamilyIPv4, BgpAddressFamilyIPv6, BgpAddressFamilyL2vpnEvpn,
    RouteMap,
    
    // Overlay related types
    Overlay, Vpc, VpcPeering, PeeringEntryFor, Expose, PeeringIPs, PeeringAs,
};

pub fn get_proto_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("proto")
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
