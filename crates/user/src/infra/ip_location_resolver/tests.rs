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
async fn ip_location_resolver_treats_public_ipv6_as_public() {
    let resolver = resolver(false, "http://127.0.0.1/unused");

    let location = resolver.resolve_login_location("2400:8d60:3::1:f2a0:9207", Locale::En).await.unwrap();

    assert_eq!(location, "Unknown");
}

#[tokio::test]
#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
async fn ip_location_resolver_returns_unknown_for_invalid_ip() {
    let resolver = resolver(true, "http://127.0.0.1/unused");

    let location = resolver.resolve_login_location("not-an-ip", Locale::En).await.unwrap();

    assert_eq!(location, "Unknown");
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

    assert_eq!(location.location, "广东省 深圳市");
    assert_eq!(location.source, SOURCE_PROVINCE_CITY);
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

    assert_eq!(location.location, "广东省 深圳市");
    assert_eq!(location.source, SOURCE_PROVINCE_CITY);
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

    assert_eq!(location.location, "美国");
    assert_eq!(location.source, SOURCE_ADDR);
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

    assert_eq!(location.location, "未知");
    assert_eq!(location.source, SOURCE_UNKNOWN);
}

fn resolver(enabled: bool, endpoint: &str) -> PconlineIpLocationResolver {
    PconlineIpLocationResolver::with_client(Arc::new(TestSettings { enabled }), reqwest::Client::new(), endpoint.into())
}
