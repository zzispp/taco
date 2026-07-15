use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use serde::Deserialize;

use crate::{ClientInfoError, ClientInfoResult};

mod pconline;
pub use pconline::PconlineIpLocationResolver;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpLocation {
    Resolved(String),
    Internal,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IpLocationConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IpLocationClientConfig {
    pub request_timeout: Duration,
}

/// Reads the operational switch that controls public IP provider lookups.
#[async_trait]
pub trait IpLocationSettingsReader: Send + Sync + 'static {
    /// Returns validated runtime settings or exposes the owning config failure.
    async fn ip_location_config(&self) -> ClientInfoResult<IpLocationConfig>;
}

/// Resolves an IP address without translating semantic location states.
///
/// Implementations return errors for configuration, transport, and parsing
/// failures. `Unknown` is reserved for successfully classified unknown data.
#[async_trait]
pub trait IpLocationResolver: Send + Sync + 'static {
    async fn resolve_ip_location(&self, ip_address: &str) -> ClientInfoResult<IpLocation>;
}

pub fn parse_ip_location_config(value: &str) -> ClientInfoResult<IpLocationConfig> {
    serde_json::from_str(value).map_err(|_| ClientInfoError::InvalidConfig(constants::system_config::IP_LOCATION_CONFIG_KEY))
}

pub(crate) type SharedSettings = Arc<dyn IpLocationSettingsReader>;

#[cfg(test)]
mod tests;
