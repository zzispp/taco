use std::sync::Arc;

use async_trait::async_trait;
use client_info::{ClientInfoError, ClientInfoResult, IpLocation, IpLocationResolver};

use super::TEST_LOGIN_LOCATION;

#[derive(Clone, Copy)]
pub(crate) enum TestIpLocationOutcome {
    Resolved,
    ProviderFailure,
    InvalidConfig,
    InvalidSetting,
    RuntimeConfig,
}

pub(crate) fn test_ip_location_resolver(outcome: TestIpLocationOutcome) -> Arc<dyn IpLocationResolver> {
    Arc::new(TestIpLocationResolver { outcome })
}

struct TestIpLocationResolver {
    outcome: TestIpLocationOutcome,
}

#[async_trait]
impl IpLocationResolver for TestIpLocationResolver {
    async fn resolve_ip_location(&self, _ipaddr: &str) -> ClientInfoResult<IpLocation> {
        match self.outcome {
            TestIpLocationOutcome::Resolved => Ok(IpLocation::Resolved(TEST_LOGIN_LOCATION.into())),
            TestIpLocationOutcome::ProviderFailure => Err(ClientInfoError::Provider("all providers failed".into())),
            TestIpLocationOutcome::InvalidConfig => Err(ClientInfoError::InvalidConfig(constants::system_config::IP_LOCATION_CONFIG_KEY)),
            TestIpLocationOutcome::InvalidSetting => Err(ClientInfoError::InvalidSetting("client_info.ip_location.request_timeout_ms")),
            TestIpLocationOutcome::RuntimeConfig => Err(ClientInfoError::RuntimeConfig("system config unavailable".into())),
        }
    }
}
