use std::{
    fmt::Display,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::Arc,
};

use async_trait::async_trait;
use serde::Deserialize;

use crate::application::{AppResult, IpLocationResolver, IpLocationSettingsReader};
use types::http::{Locale, translate_message};

const PCONLINE_IP_URL: &str = "https://whois.pconline.com.cn/ipJson.jsp";
const UNKNOWN_LOCATION_KEY: &str = "messages.user.login_location.unknown";
const INTERNAL_LOCATION_KEY: &str = "messages.user.login_location.internal";
const GBK_ENCODING: &str = "GBK";
const SOURCE_PROVINCE_CITY: &str = "province_city";
const SOURCE_REGION: &str = "region";
const SOURCE_REGION_NAMES: &str = "region_names";
const SOURCE_ADDR: &str = "addr";
const SOURCE_UNKNOWN: &str = "unknown";

#[derive(Clone)]
pub struct PconlineIpLocationResolver {
    settings: Arc<dyn IpLocationSettingsReader>,
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

impl Display for IpAddressClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => f.write_str("public"),
            Self::Internal => f.write_str("internal"),
            Self::Invalid => f.write_str("invalid"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ResolvedIpLocation {
    location: String,
    response_ip: String,
    source: &'static str,
    provider_error: String,
}

impl PconlineIpLocationResolver {
    pub fn new(settings: Arc<dyn IpLocationSettingsReader>) -> Self {
        Self::with_client(settings, reqwest::Client::new(), PCONLINE_IP_URL.into())
    }

    fn with_client(settings: Arc<dyn IpLocationSettingsReader>, client: reqwest::Client, endpoint: String) -> Self {
        Self { settings, client, endpoint }
    }
}

#[async_trait]
impl IpLocationResolver for PconlineIpLocationResolver {
    async fn resolve_login_location(&self, ipaddr: &str, locale: Locale) -> AppResult<String> {
        let ip_class = classify_ip_address(ipaddr);
        log_ip_class(ipaddr, ip_class);
        if ip_class == IpAddressClass::Internal {
            return Ok(localized_location(locale, INTERNAL_LOCATION_KEY));
        }
        if ip_class == IpAddressClass::Invalid {
            return Ok(localized_location(locale, UNKNOWN_LOCATION_KEY));
        }
        let config = self.settings.ip_location_config().await?;
        if !config.enabled {
            hook_tracing::info_with_fields!("ip location lookup disabled", ipaddr = ipaddr);
            return Ok(localized_location(locale, UNKNOWN_LOCATION_KEY));
        }
        Ok(self
            .lookup(ipaddr, locale)
            .await
            .map(|resolved| log_resolved_location(ipaddr, resolved))
            .unwrap_or_else(|error| log_lookup_error(error, locale, ipaddr)))
    }
}

impl PconlineIpLocationResolver {
    async fn lookup(&self, ipaddr: &str, locale: Locale) -> Result<ResolvedIpLocation, IpLocationLookupError> {
        let response = self
            .client
            .get(&self.endpoint)
            .query(&[("ip", ipaddr), ("json", "true")])
            .header(reqwest::header::ACCEPT_CHARSET, GBK_ENCODING)
            .send()
            .await?
            .bytes()
            .await?;
        hook_tracing::info_with_fields!("ip location provider response received", ipaddr = ipaddr, bytes = response.len());
        parse_pconline_response(&response, locale)
    }
}

#[derive(Debug, thiserror::Error)]
enum IpLocationLookupError {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("json parse failed: {0}")]
    Json(#[from] serde_json::Error),
}

fn log_lookup_error(error: IpLocationLookupError, locale: Locale, ipaddr: &str) -> String {
    hook_tracing::error_with_fields!("ip location lookup failed", &error, ipaddr = ipaddr);
    localized_location(locale, UNKNOWN_LOCATION_KEY)
}

fn log_resolved_location(ipaddr: &str, resolved: ResolvedIpLocation) -> String {
    if resolved.source == SOURCE_UNKNOWN {
        hook_tracing::warn_with_fields!(
            "ip location provider returned empty location",
            ipaddr = ipaddr,
            response_ip = resolved.response_ip,
            provider_error = resolved.provider_error,
        );
    } else {
        hook_tracing::info_with_fields!(
            "ip location resolved",
            ipaddr = ipaddr,
            response_ip = resolved.response_ip,
            source = resolved.source,
            location = resolved.location,
        );
    }
    resolved.location
}

fn log_ip_class(ipaddr: &str, ip_class: IpAddressClass) {
    if ip_class == IpAddressClass::Invalid {
        hook_tracing::warn_with_fields!("invalid login ip address", ipaddr = ipaddr);
        return;
    }
    hook_tracing::info_with_fields!("login ip address classified", ipaddr = ipaddr, class = ip_class);
}

fn parse_pconline_response(response: &[u8], locale: Locale) -> Result<ResolvedIpLocation, IpLocationLookupError> {
    let (body, _, _) = encoding_rs::GBK.decode(response);
    let parsed = serde_json::from_str::<PconlineAddressResponse>(&body)?;
    Ok(format_location(parsed, locale))
}

fn format_location(response: PconlineAddressResponse, locale: Locale) -> ResolvedIpLocation {
    let region = response.pro.trim();
    let city = response.city.trim();
    let (location, source) = match (region.is_empty(), city.is_empty()) {
        (true, true) => secondary_provider_location(&response, locale),
        (false, true) => (region.into(), SOURCE_PROVINCE_CITY),
        (true, false) => (city.into(), SOURCE_PROVINCE_CITY),
        (false, false) => (format!("{region} {city}"), SOURCE_PROVINCE_CITY),
    };
    ResolvedIpLocation {
        location,
        response_ip: response_ip(response.ip),
        source,
        provider_error: response.err,
    }
}

fn secondary_provider_location(response: &PconlineAddressResponse, locale: Locale) -> (String, &'static str) {
    first_non_empty(&[
        (response.region.as_str(), SOURCE_REGION),
        (response.region_names.as_str(), SOURCE_REGION_NAMES),
        (response.addr.as_str(), SOURCE_ADDR),
    ])
    .unwrap_or_else(|| (localized_location(locale, UNKNOWN_LOCATION_KEY), SOURCE_UNKNOWN))
}

fn first_non_empty(values: &[(&str, &'static str)]) -> Option<(String, &'static str)> {
    values
        .iter()
        .map(|(value, source)| (value.trim().to_owned(), *source))
        .find(|(value, _)| !value.is_empty())
}

fn response_ip(ip: String) -> String {
    let trimmed = ip.trim();
    if trimmed.is_empty() { "-" } else { trimmed }.into()
}

fn localized_location(locale: Locale, key: &str) -> String {
    translate_message(locale, key)
}

fn classify_ip_address(ipaddr: &str) -> IpAddressClass {
    match ipaddr.parse::<IpAddr>() {
        Ok(ip) if private_or_local(ip) => IpAddressClass::Internal,
        Ok(_) => IpAddressClass::Public,
        Err(_) => IpAddressClass::Invalid,
    }
}

fn private_or_local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => private_or_local_v4(ip),
        IpAddr::V6(ip) => private_or_local_v6(ip),
    }
}

fn private_or_local_v4(ip: Ipv4Addr) -> bool {
    ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified()
}

fn private_or_local_v6(ip: Ipv6Addr) -> bool {
    ip.is_loopback() || ip.is_unique_local() || ip.is_unicast_link_local() || ip.is_unspecified()
}

#[cfg(test)]
mod tests;
