use std::{net::Ipv4Addr, sync::Arc};

use async_trait::async_trait;
use serde::Deserialize;

use crate::application::{AppResult, IpLocationResolver, IpLocationSettingsReader};
use types::http::{Locale, translate_message};

const PCONLINE_IP_URL: &str = "https://whois.pconline.com.cn/ipJson.jsp";
const UNKNOWN_LOCATION_KEY: &str = "messages.user.login_location.unknown";
const INTERNAL_LOCATION_KEY: &str = "messages.user.login_location.internal";
const GBK_ENCODING: &str = "GBK";

#[derive(Clone)]
pub struct PconlineIpLocationResolver {
    settings: Arc<dyn IpLocationSettingsReader>,
    client: reqwest::Client,
    endpoint: String,
}

#[derive(Debug, Default, Deserialize)]
struct PconlineAddressResponse {
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
        if internal_ip(ipaddr) {
            return Ok(localized_location(locale, INTERNAL_LOCATION_KEY));
        }
        if !self.settings.ip_location_config().await?.enabled {
            return Ok(localized_location(locale, UNKNOWN_LOCATION_KEY));
        }
        Ok(self.lookup(ipaddr, locale).await.unwrap_or_else(|error| log_lookup_error(error, locale)))
    }
}

impl PconlineIpLocationResolver {
    async fn lookup(&self, ipaddr: &str, locale: Locale) -> Result<String, IpLocationLookupError> {
        let response = self
            .client
            .get(&self.endpoint)
            .query(&[("ip", ipaddr), ("json", "true")])
            .header(reqwest::header::ACCEPT_CHARSET, GBK_ENCODING)
            .send()
            .await?
            .bytes()
            .await?;
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

fn log_lookup_error(error: IpLocationLookupError, locale: Locale) -> String {
    hook_tracing::error("ip location lookup failed", &error);
    localized_location(locale, UNKNOWN_LOCATION_KEY)
}

fn parse_pconline_response(response: &[u8], locale: Locale) -> Result<String, IpLocationLookupError> {
    let (body, _, _) = encoding_rs::GBK.decode(response);
    let parsed = serde_json::from_str::<PconlineAddressResponse>(&body)?;
    Ok(format_location(parsed, locale))
}

fn format_location(response: PconlineAddressResponse, locale: Locale) -> String {
    let region = response.pro.trim();
    let city = response.city.trim();
    match (region.is_empty(), city.is_empty()) {
        (true, true) => fallback_location(response, locale),
        (false, true) => region.into(),
        (true, false) => city.into(),
        (false, false) => format!("{region} {city}"),
    }
}

fn fallback_location(response: PconlineAddressResponse, locale: Locale) -> String {
    [response.region, response.region_names, response.addr]
        .into_iter()
        .map(|value| value.trim().to_owned())
        .find(|value| !value.is_empty())
        .unwrap_or_else(|| localized_location(locale, UNKNOWN_LOCATION_KEY))
}

fn localized_location(locale: Locale, key: &str) -> String {
    translate_message(locale, key)
}

fn internal_ip(ipaddr: &str) -> bool {
    let Ok(ip) = ipaddr.parse::<Ipv4Addr>() else {
        return true;
    };
    ip.is_private() || ip.is_loopback()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::*;
    use crate::application::IpLocationConfig;

    struct TestSettings {
        enabled: bool,
    }

    #[async_trait]
    impl IpLocationSettingsReader for TestSettings {
        async fn ip_location_config(&self) -> AppResult<IpLocationConfig> {
            Ok(IpLocationConfig { enabled: self.enabled })
        }
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
    async fn ip_location_resolver_returns_internal_location_for_private_ip() {
        let resolver = resolver(false, "http://127.0.0.1/unused");

        let location = resolver.resolve_login_location("192.168.1.10", Locale::ZhCn).await.unwrap();

        assert_eq!(location, "内网IP");
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
    async fn ip_location_resolver_returns_unknown_when_disabled() {
        let resolver = resolver(false, "http://127.0.0.1/unused");

        let location = resolver.resolve_login_location("8.8.8.8", Locale::En).await.unwrap();

        assert_eq!(location, "Unknown");
    }

    #[test]
    fn parse_pconline_response_decodes_gbk_json() {
        let (payload, _, _) = encoding_rs::GBK.encode(r#"{"pro":"广东省","city":"深圳市"}"#);

        let location = parse_pconline_response(&payload, Locale::ZhCn).unwrap();

        assert_eq!(location, "广东省 深圳市");
    }

    #[test]
    fn format_location_joins_region_and_city() {
        let location = format_location(
            PconlineAddressResponse {
                pro: "广东省".into(),
                city: "深圳市".into(),
                ..Default::default()
            },
            Locale::ZhCn,
        );

        assert_eq!(location, "广东省 深圳市");
    }

    #[test]
    fn format_location_uses_addr_when_region_and_city_are_empty() {
        let location = format_location(
            PconlineAddressResponse {
                addr: " 美国".into(),
                ..Default::default()
            },
            Locale::ZhCn,
        );

        assert_eq!(location, "美国");
    }

    #[test]
    fn format_location_returns_unknown_for_empty_response() {
        let location = format_location(
            PconlineAddressResponse {
                pro: "".into(),
                city: "".into(),
                ..Default::default()
            },
            Locale::ZhTw,
        );

        assert_eq!(location, "未知");
    }

    fn resolver(enabled: bool, endpoint: &str) -> PconlineIpLocationResolver {
        PconlineIpLocationResolver::with_client(Arc::new(TestSettings { enabled }), reqwest::Client::new(), endpoint.into())
    }
}
