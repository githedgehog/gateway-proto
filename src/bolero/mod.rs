// SPDX-License-Identifier: Apache-2.0
// Copyright 2025 Hedgehog

mod bgp;
mod device;
mod expose;
mod gateway_config;
mod impl_peering_as;
mod impl_peering_i_ps;
mod interface;
pub mod support;
#[cfg(test)]
pub mod test_support;
mod vpc;
mod vrf;

#[allow(unused)] // Currently only implements traits, remove if we export anything
pub use bgp::*;
#[allow(unused)] // Currently only implements traits, remove if we export anything
pub use device::*;
#[allow(unused)] // Currently only implements traits, remove if we export anything
pub use expose::*;
#[allow(unused)] // Currently only implements traits, remove if we export anything
pub use gateway_config::*;
pub use impl_peering_as::*;
pub use impl_peering_i_ps::*;
#[allow(unused)] // Currently only implements traits, remove if we export anything
pub use interface::*;
#[allow(unused)] // Currently only implements traits, remove if we export anything
pub use vpc::*;
#[allow(unused)] // Currently only implements traits, remove if we export anything
pub use vrf::*;
