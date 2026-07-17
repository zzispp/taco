use std::{future::pending, sync::Arc, time::Duration};

use async_trait::async_trait;

use super::{
    IpLocation, IpLocationClientConfig, IpLocationConfig, IpLocationResolver, IpLocationSettingsReader, PconlineIpLocationResolver, parse_ip_location_config,
    pconline::{SOURCE_PROVINCE_CITY, parse_pconline_response},
};
use crate::{ClientInfoError, ClientInfoResult};

struct TestSettings {
    enabled: bool,
}

#[async_trait]
impl IpLocationSettingsReader for TestSettings {
    async fn ip_location_config(&self) -> ClientInfoResult<IpLocationConfig> {
        Ok(IpLocationConfig { enabled: self.enabled })
    }
}

#[test]
fn config_requires_boolean_enabled_and_rejects_unknown_fields() {
    assert_eq!(parse_ip_location_config(r#"{"enabled":true}"#).unwrap(), IpLocationConfig { enabled: true });
    for value in [r#"{"enabled":"true"}"#, r#"{"enabled":true,"other":1}"#, "not-json"] {
        assert!(matches!(parse_ip_location_config(value), Err(ClientInfoError::InvalidConfig(_))));
    }
}

#[tokio::test]
async fn internal_invalid_and_disabled_addresses_return_semantic_states() {
    let resolver = resolver(false, "http://127.0.0.1:1", 50);

    assert_eq!(resolver.resolve_ip_location("192.168.1.10").await.unwrap(), IpLocation::Internal);
    assert_eq!(resolver.resolve_ip_location("not-an-ip").await.unwrap(), IpLocation::Unknown);
    assert_eq!(resolver.resolve_ip_location("8.8.8.8").await.unwrap(), IpLocation::Unknown);
}

#[test]
fn provider_payload_prefers_province_and_city_without_translation() {
    let (payload, _, _) = encoding_rs::GBK.encode(r#"{"ip":"8.8.8.8","pro":"广东省","city":"深圳市"}"#);

    let resolved = parse_pconline_response(&payload).unwrap();

    assert_eq!(resolved.location.as_deref(), Some("广东省 深圳市"));
    assert_eq!(resolved.source, SOURCE_PROVINCE_CITY);
}

#[test]
fn zero_http_timeout_is_rejected_explicitly() {
    let result = PconlineIpLocationResolver::new(
        Arc::new(TestSettings { enabled: true }),
        IpLocationClientConfig {
            request_timeout: Duration::ZERO,
        },
        observer(),
    );

    assert!(matches!(
        result,
        Err(ClientInfoError::InvalidSetting("client_info.ip_location.request_timeout_ms"))
    ));
}

#[tokio::test]
async fn hanging_provider_is_cut_off_by_the_http_client_timeout() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
        pending::<()>().await;
    });
    let resolver = resolver(true, &endpoint, 20);

    let result = resolver.resolve_ip_location("8.8.8.8").await;

    assert!(matches!(result, Err(ClientInfoError::Provider(_))));
    server.abort();
}

fn resolver(enabled: bool, endpoint: &str, timeout_ms: u64) -> PconlineIpLocationResolver {
    PconlineIpLocationResolver::with_endpoint(
        Arc::new(TestSettings { enabled }),
        IpLocationClientConfig {
            request_timeout: Duration::from_millis(timeout_ms),
        },
        endpoint.into(),
        observer(),
    )
    .unwrap()
}

fn observer() -> taco_tracing::InfrastructureObserver {
    let config = taco_tracing::parse_runtime_tracing_config(
        r#"{"log_level":"error","http":{"access_enabled":true,"capture_request_body":false,"capture_response_body":false,"capture_query_parameters":false,"capture_request_headers":false,"max_body_capture_bytes":0},"slow_operation_ms":{"postgres":500,"redis":100,"outbound_http":1000}}"#,
    )
    .unwrap();
    taco_tracing::InfrastructureObserver::new(taco_tracing::RuntimeTracingState::new(config))
}
