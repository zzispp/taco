use std::{fmt::Display, net::IpAddr, net::Ipv4Addr, net::Ipv6Addr, time::Instant};

use async_trait::async_trait;
use metrics::counter;
use taco_tracing::{InfrastructureDependency, InfrastructureObserver, InfrastructureOperation};

use crate::{ClientInfoError, ClientInfoResult};

use super::{
    IpLocation, IpLocationClientConfig, IpLocationResolver, SharedSettings,
    provider::{ProviderEndpoint, ProviderFailure, ProviderLocation, default_provider_endpoints, parse_provider_response},
};

const IP_LOCATION_TIMEOUT_SETTING: &str = "client_info.ip_location.request_timeout_ms";
const IP_LOCATION_LOOKUP_OPERATION: &str = "ip_location_lookup";
pub(super) const IP_LOCATION_PROVIDER_FAILURES_TOTAL: &str = "client_info_ip_location_provider_failures_total";
pub(super) const IP_LOCATION_EXHAUSTED_TOTAL: &str = "client_info_ip_location_exhausted_total";

pub(super) struct ResolverDependencies {
    providers: Vec<ProviderEndpoint>,
    observer: InfrastructureObserver,
}

impl ResolverDependencies {
    pub(super) fn new(providers: Vec<ProviderEndpoint>, observer: InfrastructureObserver) -> Self {
        Self { providers, observer }
    }
}

#[derive(Clone)]
pub struct PublicIpLocationResolver {
    settings: SharedSettings,
    client: reqwest::Client,
    providers: Vec<ProviderEndpoint>,
    observer: InfrastructureObserver,
}

impl PublicIpLocationResolver {
    pub fn new(settings: SharedSettings, config: IpLocationClientConfig, observer: InfrastructureObserver) -> ClientInfoResult<Self> {
        let dependencies = ResolverDependencies::new(default_provider_endpoints()?, observer);
        Self::with_dependencies(settings, config, dependencies)
    }

    pub(super) fn with_dependencies(settings: SharedSettings, config: IpLocationClientConfig, dependencies: ResolverDependencies) -> ClientInfoResult<Self> {
        Ok(Self {
            settings,
            client: build_client(config)?,
            providers: dependencies.providers,
            observer: dependencies.observer,
        })
    }

    async fn resolve_public_ip(&self, ip_address: &str) -> ClientInfoResult<IpLocation> {
        let mut last_failure = None;
        for (index, provider) in self.providers.iter().enumerate() {
            match self.lookup(provider, ip_address).await {
                Ok(location) => return Ok(log_resolved_location(provider, ip_address, location)),
                Err(error) => {
                    record_provider_failure(
                        ProviderAttempt {
                            provider,
                            ip_address,
                            index,
                            provider_count: self.providers.len(),
                        },
                        &error,
                    );
                    last_failure = Some((provider.kind().name(), error.to_string()));
                }
            }
        }
        Err(provider_chain_exhausted(ip_address, self.providers.len(), last_failure))
    }

    async fn lookup(&self, provider: &ProviderEndpoint, ip_address: &str) -> Result<ProviderLocation, ProviderFailure> {
        let started = Instant::now();
        let result = self.request_and_parse(provider, ip_address).await;
        self.observer.record(InfrastructureOperation {
            dependency: InfrastructureDependency::OutboundHttp,
            operation: IP_LOCATION_LOOKUP_OPERATION,
            elapsed: started.elapsed(),
            succeeded: result.is_ok(),
        });
        result
    }

    async fn request_and_parse(&self, provider: &ProviderEndpoint, ip_address: &str) -> Result<ProviderLocation, ProviderFailure> {
        let url = provider.request_url(ip_address)?;
        let response = self.client.get(url).send().await.map_err(ProviderFailure::Request)?;
        if !response.status().is_success() {
            return Err(ProviderFailure::HttpStatus(response.status()));
        }
        let body = response.bytes().await.map_err(ProviderFailure::Request)?;
        parse_provider_response(provider.kind(), &body, ip_address)
    }
}

#[async_trait]
impl IpLocationResolver for PublicIpLocationResolver {
    async fn resolve_ip_location(&self, ip_address: &str) -> ClientInfoResult<IpLocation> {
        let address_class = classify_ip_address(ip_address);
        log_ip_class(ip_address, address_class);
        match address_class {
            IpAddressClass::Internal => return Ok(IpLocation::Internal),
            IpAddressClass::Invalid => return Ok(IpLocation::Unknown),
            IpAddressClass::Public => {}
        }
        if !self.settings.ip_location_config().await?.enabled {
            taco_tracing::info_with_fields!("ip location lookup disabled", ip_address = ip_address);
            return Ok(IpLocation::Unknown);
        }
        self.resolve_public_ip(ip_address).await
    }
}

fn build_client(config: IpLocationClientConfig) -> ClientInfoResult<reqwest::Client> {
    if config.request_timeout.is_zero() {
        return Err(ClientInfoError::InvalidSetting(IP_LOCATION_TIMEOUT_SETTING));
    }
    reqwest::Client::builder()
        .timeout(config.request_timeout)
        .connect_timeout(config.request_timeout)
        .read_timeout(config.request_timeout)
        .build()
        .map_err(provider_error)
}

struct ProviderAttempt<'a> {
    provider: &'a ProviderEndpoint,
    ip_address: &'a str,
    index: usize,
    provider_count: usize,
}

fn record_provider_failure(attempt: ProviderAttempt<'_>, error: &ProviderFailure) {
    counter!(
        IP_LOCATION_PROVIDER_FAILURES_TOTAL,
        "provider" => attempt.provider.kind().name(),
        "reason" => error.category()
    )
    .increment(1);
    taco_tracing::warn_with_fields!(
        "ip location provider failed",
        provider = attempt.provider.kind().name(),
        reason = error.category(),
        http_status = error.status(),
        attempt = attempt.index + 1,
        provider_count = attempt.provider_count,
        ip_address = attempt.ip_address,
        error = error,
    );
}

fn provider_chain_exhausted(ip_address: &str, provider_count: usize, last_failure: Option<(&'static str, String)>) -> ClientInfoError {
    counter!(IP_LOCATION_EXHAUSTED_TOTAL).increment(1);
    let (last_provider, last_error) = last_failure.unwrap_or(("none", "provider chain is empty".into()));
    let error = format!("all {provider_count} IP location providers failed; last provider {last_provider}: {last_error}");
    taco_tracing::error_with_fields!(
        "ip location provider chain exhausted",
        &error,
        ip_address = ip_address,
        provider_count = provider_count,
        last_provider = last_provider,
    );
    ClientInfoError::Provider(error)
}

fn log_resolved_location(provider: &ProviderEndpoint, ip_address: &str, resolved: ProviderLocation) -> IpLocation {
    taco_tracing::info_with_fields!(
        "ip location resolved",
        provider = provider.kind().name(),
        ip_address = ip_address,
        response_ip = resolved.response_ip,
        location = resolved.location,
    );
    IpLocation::Resolved(resolved.location)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IpAddressClass {
    Public,
    Internal,
    Invalid,
}

impl Display for IpAddressClass {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Public => "public",
            Self::Internal => "internal",
            Self::Invalid => "invalid",
        })
    }
}

fn log_ip_class(ip_address: &str, address_class: IpAddressClass) {
    if address_class == IpAddressClass::Invalid {
        taco_tracing::warn_with_fields!("invalid client IP address", ip_address = ip_address);
        return;
    }
    taco_tracing::info_with_fields!("client IP address classified", ip_address = ip_address, class = address_class);
}

fn classify_ip_address(ip_address: &str) -> IpAddressClass {
    match ip_address.parse::<IpAddr>() {
        Ok(address) if private_or_local(address) => IpAddressClass::Internal,
        Ok(_) => IpAddressClass::Public,
        Err(_) => IpAddressClass::Invalid,
    }
}

fn private_or_local(ip_address: IpAddr) -> bool {
    match ip_address {
        IpAddr::V4(address) => private_or_local_v4(address),
        IpAddr::V6(address) => private_or_local_v6(address),
    }
}

fn private_or_local_v4(ip_address: Ipv4Addr) -> bool {
    ip_address.is_private() || ip_address.is_loopback() || ip_address.is_link_local() || ip_address.is_unspecified()
}

fn private_or_local_v6(ip_address: Ipv6Addr) -> bool {
    ip_address.is_loopback() || ip_address.is_unique_local() || ip_address.is_unicast_link_local() || ip_address.is_unspecified()
}

fn provider_error(error: impl Display) -> ClientInfoError {
    ClientInfoError::Provider(error.to_string())
}
