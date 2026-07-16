use std::{
    fmt::Display,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use async_trait::async_trait;
use serde::Deserialize;

use crate::{ClientInfoError, ClientInfoResult};

use super::{IpLocation, IpLocationClientConfig, IpLocationResolver, SharedSettings};

const PCONLINE_IP_URL: &str = "https://whois.pconline.com.cn/ipJson.jsp";
const GBK_ENCODING: &str = "GBK";
pub(super) const SOURCE_PROVINCE_CITY: &str = "province_city";
const SOURCE_REGION: &str = "region";
const SOURCE_REGION_NAMES: &str = "region_names";
const SOURCE_ADDR: &str = "addr";
const SOURCE_UNKNOWN: &str = "unknown";

#[derive(Clone)]
pub struct PconlineIpLocationResolver {
    settings: SharedSettings,
    client: reqwest::Client,
    endpoint: String,
}

#[derive(Debug, Default, Deserialize)]
struct PconlineAddressResponse {
    #[serde(default)]
    ip: String,
    #[serde(default)]
    pro: String,
    #[serde(default)]
    city: String,
    #[serde(default)]
    region: String,
    #[serde(default, rename = "regionNames")]
    region_names: String,
    #[serde(default)]
    addr: String,
    #[serde(default)]
    err: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IpAddressClass {
    Public,
    Internal,
    Invalid,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ProviderLocation {
    pub(super) location: Option<String>,
    response_ip: String,
    pub(super) source: &'static str,
    provider_error: String,
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

impl PconlineIpLocationResolver {
    pub fn new(settings: SharedSettings, config: IpLocationClientConfig) -> ClientInfoResult<Self> {
        let client = build_client(config)?;
        Ok(Self {
            settings,
            client,
            endpoint: PCONLINE_IP_URL.into(),
        })
    }

    #[cfg(test)]
    pub(super) fn with_endpoint(settings: SharedSettings, config: IpLocationClientConfig, endpoint: String) -> ClientInfoResult<Self> {
        Ok(Self {
            settings,
            client: build_client(config)?,
            endpoint,
        })
    }

    async fn lookup(&self, ip_address: &str) -> ClientInfoResult<ProviderLocation> {
        let response = self
            .client
            .get(&self.endpoint)
            .query(&[("ip", ip_address), ("json", "true")])
            .header(reqwest::header::ACCEPT_CHARSET, GBK_ENCODING)
            .send()
            .await
            .map_err(provider_error)?
            .error_for_status()
            .map_err(provider_error)?
            .bytes()
            .await
            .map_err(provider_error)?;
        taco_tracing::info_with_fields!("ip location provider response received", ip_address = ip_address, bytes = response.len());
        parse_pconline_response(&response)
    }
}

#[async_trait]
impl IpLocationResolver for PconlineIpLocationResolver {
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
        self.lookup(ip_address).await.map(|resolved| log_resolved_location(ip_address, resolved))
    }
}

fn build_client(config: IpLocationClientConfig) -> ClientInfoResult<reqwest::Client> {
    if config.request_timeout.is_zero() {
        return Err(ClientInfoError::InvalidSetting("client_info.ip_location.request_timeout_ms"));
    }
    reqwest::Client::builder()
        .timeout(config.request_timeout)
        .connect_timeout(config.request_timeout)
        .read_timeout(config.request_timeout)
        .build()
        .map_err(provider_error)
}

fn log_resolved_location(ip_address: &str, resolved: ProviderLocation) -> IpLocation {
    let Some(location) = resolved.location else {
        taco_tracing::warn_with_fields!(
            "ip location provider returned empty location",
            ip_address = ip_address,
            response_ip = resolved.response_ip,
            provider_error = resolved.provider_error,
        );
        return IpLocation::Unknown;
    };
    taco_tracing::info_with_fields!(
        "ip location resolved",
        ip_address = ip_address,
        response_ip = resolved.response_ip,
        source = resolved.source,
        location = location,
    );
    IpLocation::Resolved(location)
}

fn log_ip_class(ip_address: &str, address_class: IpAddressClass) {
    if address_class == IpAddressClass::Invalid {
        taco_tracing::warn_with_fields!("invalid client IP address", ip_address = ip_address);
        return;
    }
    taco_tracing::info_with_fields!("client IP address classified", ip_address = ip_address, class = address_class);
}

pub(super) fn parse_pconline_response(response: &[u8]) -> ClientInfoResult<ProviderLocation> {
    let (body, _, _) = encoding_rs::GBK.decode(response);
    serde_json::from_str::<PconlineAddressResponse>(&body)
        .map(format_location)
        .map_err(provider_error)
}

fn format_location(response: PconlineAddressResponse) -> ProviderLocation {
    let region = response.pro.trim();
    let city = response.city.trim();
    let (location, source) = match (region.is_empty(), city.is_empty()) {
        (true, true) => secondary_provider_location(&response),
        (false, true) => (Some(region.into()), SOURCE_PROVINCE_CITY),
        (true, false) => (Some(city.into()), SOURCE_PROVINCE_CITY),
        (false, false) => (Some(format!("{region} {city}")), SOURCE_PROVINCE_CITY),
    };
    ProviderLocation {
        location,
        response_ip: response_ip(response.ip),
        source,
        provider_error: response.err,
    }
}

fn secondary_provider_location(response: &PconlineAddressResponse) -> (Option<String>, &'static str) {
    first_non_empty(&[
        (response.region.as_str(), SOURCE_REGION),
        (response.region_names.as_str(), SOURCE_REGION_NAMES),
        (response.addr.as_str(), SOURCE_ADDR),
    ])
    .map_or((None, SOURCE_UNKNOWN), |(value, source)| (Some(value), source))
}

fn first_non_empty(values: &[(&str, &'static str)]) -> Option<(String, &'static str)> {
    values
        .iter()
        .map(|(value, source)| (value.trim().to_owned(), *source))
        .find(|(value, _)| !value.is_empty())
}

fn response_ip(ip_address: String) -> String {
    let trimmed = ip_address.trim();
    if trimmed.is_empty() { "-" } else { trimmed }.into()
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
