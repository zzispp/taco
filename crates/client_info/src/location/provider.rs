use std::net::IpAddr;

use reqwest::{StatusCode, Url};
use serde::Deserialize;
use thiserror::Error;

use crate::{ClientInfoError, ClientInfoResult};

const IPWHO_IS_ENDPOINT: &str = "https://ipwho.is/";
const IPQUERY_ENDPOINT: &str = "https://api.ipquery.io/";
const GEOJS_ENDPOINT: &str = "https://get.geojs.io/v1/ip/geo/";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProviderKind {
    IpWhoIs,
    IpQuery,
    GeoJs,
}

impl ProviderKind {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::IpWhoIs => "ipwho_is",
            Self::IpQuery => "ipquery_io",
            Self::GeoJs => "geojs",
        }
    }

    fn ip_path(self, ip_address: &str) -> String {
        match self {
            Self::GeoJs => format!("{ip_address}.json"),
            Self::IpWhoIs | Self::IpQuery => ip_address.to_owned(),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct ProviderEndpoint {
    kind: ProviderKind,
    base_url: Url,
}

impl ProviderEndpoint {
    pub(super) fn new(kind: ProviderKind, base_url: &str) -> ClientInfoResult<Self> {
        let base_url = Url::parse(base_url).map_err(|error| ClientInfoError::Provider(format!("invalid {} endpoint: {error}", kind.name())))?;
        Ok(Self { kind, base_url })
    }

    pub(super) fn kind(&self) -> ProviderKind {
        self.kind
    }

    pub(super) fn request_url(&self, ip_address: &str) -> Result<Url, ProviderFailure> {
        let mut url = self.base_url.clone();
        let path = self.kind.ip_path(ip_address);
        url.path_segments_mut()
            .map_err(|_| ProviderFailure::InvalidRequest("endpoint cannot contain path segments".into()))?
            .pop_if_empty()
            .push(&path);
        Ok(url)
    }
}

pub(super) fn default_provider_endpoints() -> ClientInfoResult<Vec<ProviderEndpoint>> {
    Ok(vec![
        ProviderEndpoint::new(ProviderKind::IpWhoIs, IPWHO_IS_ENDPOINT)?,
        ProviderEndpoint::new(ProviderKind::IpQuery, IPQUERY_ENDPOINT)?,
        ProviderEndpoint::new(ProviderKind::GeoJs, GEOJS_ENDPOINT)?,
    ])
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ProviderLocation {
    pub(super) location: String,
    pub(super) response_ip: String,
}

#[derive(Debug, Error)]
pub(super) enum ProviderFailure {
    #[error("request failed: {0}")]
    Request(#[source] reqwest::Error),
    #[error("provider returned HTTP {0}")]
    HttpStatus(StatusCode),
    #[error("provider response is invalid: {0}")]
    InvalidResponse(String),
    #[error("provider rejected the request: {0}")]
    ProviderRejected(String),
    #[error("provider returned no country")]
    EmptyLocation,
    #[error("provider request URL is invalid: {0}")]
    InvalidRequest(String),
}

impl ProviderFailure {
    pub(super) const fn category(&self) -> &'static str {
        match self {
            Self::Request(_) => "request",
            Self::HttpStatus(_) => "http_status",
            Self::InvalidResponse(_) => "invalid_response",
            Self::ProviderRejected(_) => "provider_rejected",
            Self::EmptyLocation => "empty_location",
            Self::InvalidRequest(_) => "invalid_request",
        }
    }

    pub(super) fn status(&self) -> String {
        match self {
            Self::HttpStatus(status) => status.as_u16().to_string(),
            _ => "-".into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct IpWhoIsResponse {
    ip: String,
    success: Option<bool>,
    #[serde(default)]
    message: String,
    #[serde(default)]
    country: String,
    #[serde(default)]
    region: String,
    #[serde(default)]
    city: String,
}

#[derive(Debug, Deserialize)]
struct IpQueryResponse {
    ip: String,
    location: IpQueryLocation,
}

#[derive(Debug, Deserialize)]
struct IpQueryLocation {
    #[serde(default)]
    country: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    city: String,
}

#[derive(Debug, Deserialize)]
struct GeoJsResponse {
    ip: String,
    #[serde(default)]
    country: String,
    #[serde(default)]
    region: String,
    #[serde(default)]
    city: String,
}

pub(super) fn parse_provider_response(kind: ProviderKind, body: &[u8], expected_ip: &str) -> Result<ProviderLocation, ProviderFailure> {
    match kind {
        ProviderKind::IpWhoIs => parse_ipwho_is(body, expected_ip),
        ProviderKind::IpQuery => parse_ipquery(body, expected_ip),
        ProviderKind::GeoJs => parse_geojs(body, expected_ip),
    }
}

fn parse_ipwho_is(body: &[u8], expected_ip: &str) -> Result<ProviderLocation, ProviderFailure> {
    let response: IpWhoIsResponse = parse_json(body)?;
    if response.success != Some(true) {
        let message = if response.message.trim().is_empty() {
            "success flag is missing or false".into()
        } else {
            response.message
        };
        return Err(ProviderFailure::ProviderRejected(message));
    }
    validated_location(expected_ip, response.ip, [&response.country, &response.region, &response.city])
}

fn parse_ipquery(body: &[u8], expected_ip: &str) -> Result<ProviderLocation, ProviderFailure> {
    let response: IpQueryResponse = parse_json(body)?;
    validated_location(
        expected_ip,
        response.ip,
        [&response.location.country, &response.location.state, &response.location.city],
    )
}

fn parse_geojs(body: &[u8], expected_ip: &str) -> Result<ProviderLocation, ProviderFailure> {
    let response: GeoJsResponse = parse_json(body)?;
    validated_location(expected_ip, response.ip, [&response.country, &response.region, &response.city])
}

fn parse_json<'a, T: Deserialize<'a>>(body: &'a [u8]) -> Result<T, ProviderFailure> {
    serde_json::from_slice(body).map_err(|error| ProviderFailure::InvalidResponse(error.to_string()))
}

fn validated_location(expected_ip: &str, response_ip: String, parts: [&str; 3]) -> Result<ProviderLocation, ProviderFailure> {
    validate_response_ip(expected_ip, &response_ip)?;
    let [country, region, city] = parts;
    let country = country.trim();
    if country.is_empty() {
        return Err(ProviderFailure::EmptyLocation);
    }
    Ok(ProviderLocation {
        location: join_location(country, region, city),
        response_ip,
    })
}

fn validate_response_ip(expected_ip: &str, response_ip: &str) -> Result<(), ProviderFailure> {
    let expected = expected_ip
        .parse::<IpAddr>()
        .map_err(|_| ProviderFailure::InvalidResponse("requested IP is invalid".into()))?;
    let actual = response_ip
        .trim()
        .parse::<IpAddr>()
        .map_err(|_| ProviderFailure::InvalidResponse("response IP is missing or invalid".into()))?;
    if actual != expected {
        return Err(ProviderFailure::InvalidResponse("response IP does not match the request".into()));
    }
    Ok(())
}

fn join_location(country: &str, region: &str, city: &str) -> String {
    let mut parts = Vec::with_capacity(3);
    for part in [country, region, city].map(str::trim) {
        if !part.is_empty() && parts.last().copied() != Some(part) {
            parts.push(part);
        }
    }
    parts.join(" ")
}
