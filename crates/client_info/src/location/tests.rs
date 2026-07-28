use std::{future::pending, sync::Arc, time::Duration};

use async_trait::async_trait;
use metrics::set_default_local_recorder;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::{
    IpLocation, IpLocationClientConfig, IpLocationConfig, IpLocationResolver, IpLocationSettingsReader, PublicIpLocationResolver, parse_ip_location_config,
    provider::{ProviderEndpoint, ProviderKind, parse_provider_response},
    resolver::{IP_LOCATION_EXHAUSTED_TOTAL, IP_LOCATION_PROVIDER_FAILURES_TOTAL, ResolverDependencies},
};
use crate::{ClientInfoError, ClientInfoResult};

const TEST_IP: &str = "8.8.8.8";

struct TestSettings {
    result: ClientInfoResult<IpLocationConfig>,
}

#[async_trait]
impl IpLocationSettingsReader for TestSettings {
    async fn ip_location_config(&self) -> ClientInfoResult<IpLocationConfig> {
        match &self.result {
            Ok(config) => Ok(config.clone()),
            Err(ClientInfoError::InvalidConfig(key)) => Err(ClientInfoError::InvalidConfig(key)),
            Err(ClientInfoError::InvalidSetting(key)) => Err(ClientInfoError::InvalidSetting(key)),
            Err(ClientInfoError::RuntimeConfig(message)) => Err(ClientInfoError::RuntimeConfig(message.clone())),
            Err(ClientInfoError::Provider(message)) => Err(ClientInfoError::Provider(message.clone())),
        }
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
    let resolver = resolver(false, vec![provider(ProviderKind::IpWhoIs, "http://127.0.0.1:1/")], 50);

    assert_eq!(resolver.resolve_ip_location("192.168.1.10").await.unwrap(), IpLocation::Internal);
    assert_eq!(resolver.resolve_ip_location("not-an-ip").await.unwrap(), IpLocation::Unknown);
    assert_eq!(resolver.resolve_ip_location(TEST_IP).await.unwrap(), IpLocation::Unknown);
}

#[tokio::test]
async fn runtime_config_failures_bypass_the_provider_chain() {
    let resolver = PublicIpLocationResolver::with_dependencies(
        Arc::new(TestSettings {
            result: Err(ClientInfoError::RuntimeConfig("system config unavailable".into())),
        }),
        IpLocationClientConfig {
            request_timeout: Duration::from_millis(50),
        },
        ResolverDependencies::new(vec![provider(ProviderKind::IpWhoIs, "http://127.0.0.1:1/")], observer()),
    )
    .unwrap();

    let result = resolver.resolve_ip_location(TEST_IP).await;

    assert!(matches!(result, Err(ClientInfoError::RuntimeConfig(message)) if message == "system config unavailable"));
}

#[test]
fn provider_payload_requires_success_and_a_country() {
    let rejected = br#"{"ip":"8.8.8.8","success":false,"message":"rate limited"}"#;
    let empty = br#"{"ip":"8.8.8.8","success":true,"country":"","region":"","city":""}"#;

    assert_eq!(
        parse_provider_response(ProviderKind::IpWhoIs, rejected, TEST_IP).unwrap_err().category(),
        "provider_rejected"
    );
    assert_eq!(
        parse_provider_response(ProviderKind::IpWhoIs, empty, TEST_IP).unwrap_err().category(),
        "empty_location"
    );
}

#[test]
fn successful_provider_payloads_use_the_same_location_shape() {
    let fixtures = [
        (
            ProviderKind::IpWhoIs,
            br#"{"ip":"8.8.8.8","success":true,"country":"United States","region":"California","city":"Mountain View"}"#.as_slice(),
        ),
        (
            ProviderKind::IpQuery,
            br#"{"ip":"8.8.8.8","location":{"country":"United States","state":"California","city":"Mountain View"}}"#.as_slice(),
        ),
        (
            ProviderKind::GeoJs,
            br#"{"ip":"8.8.8.8","country":"United States","region":"California","city":"Mountain View"}"#.as_slice(),
        ),
    ];

    for (provider, payload) in fixtures {
        let resolved = parse_provider_response(provider, payload, TEST_IP).unwrap();
        assert_eq!(resolved.location, "United States California Mountain View");
        assert_eq!(resolved.response_ip, TEST_IP);
    }
}

#[test]
fn provider_urls_preserve_ipv6_as_one_path_segment() {
    let provider = ProviderEndpoint::new(ProviderKind::GeoJs, "https://get.geojs.io/v1/ip/geo/").unwrap();

    let url = provider.request_url("2001:4860:4860::8888").unwrap();

    assert_eq!(url.as_str(), "https://get.geojs.io/v1/ip/geo/2001:4860:4860::8888.json");
}

#[tokio::test]
async fn failed_provider_falls_through_to_the_next_provider() {
    let failed = serve_once("503 Service Unavailable", "unavailable").await;
    let successful = serve_once(
        "200 OK",
        r#"{"ip":"8.8.8.8","location":{"country":"United States","state":"California","city":"Mountain View"}}"#,
    )
    .await;
    let resolver = resolver(
        true,
        vec![provider(ProviderKind::IpWhoIs, &failed), provider(ProviderKind::IpQuery, &successful)],
        100,
    );

    let location = resolver.resolve_ip_location(TEST_IP).await.unwrap();

    assert_eq!(location, IpLocation::Resolved("United States California Mountain View".into()));
}

#[tokio::test]
async fn invalid_json_falls_through_to_the_next_provider() {
    let invalid = serve_once("200 OK", "not-json").await;
    let successful = serve_once(
        "200 OK",
        r#"{"ip":"8.8.8.8","country":"United States","region":"California","city":"Mountain View"}"#,
    )
    .await;
    let resolver = resolver(
        true,
        vec![provider(ProviderKind::IpQuery, &invalid), provider(ProviderKind::GeoJs, &successful)],
        100,
    );

    let location = resolver.resolve_ip_location(TEST_IP).await.unwrap();

    assert_eq!(location, IpLocation::Resolved("United States California Mountain View".into()));
}

#[tokio::test(flavor = "current_thread")]
async fn exhausted_provider_chain_records_each_failure_and_the_final_failure() {
    let recorder = PrometheusBuilder::new().build_recorder();
    let handle = recorder.handle();
    let _guard = set_default_local_recorder(&recorder);
    let invalid = serve_once("200 OK", "not-json").await;
    let unavailable = serve_once("503 Service Unavailable", "unavailable").await;
    let resolver = resolver(
        true,
        vec![provider(ProviderKind::IpWhoIs, &invalid), provider(ProviderKind::IpQuery, &unavailable)],
        100,
    );

    let result = resolver.resolve_ip_location(TEST_IP).await;

    assert!(matches!(result, Err(ClientInfoError::Provider(_))));
    let rendered = handle.render();
    assert_metric(
        &rendered,
        &format!(r#"{IP_LOCATION_PROVIDER_FAILURES_TOTAL}{{provider="ipwho_is",reason="invalid_response"}}"#),
        1,
    );
    assert_metric(
        &rendered,
        &format!(r#"{IP_LOCATION_PROVIDER_FAILURES_TOTAL}{{provider="ipquery_io",reason="http_status"}}"#),
        1,
    );
    assert_metric(&rendered, IP_LOCATION_EXHAUSTED_TOTAL, 1);
}

#[test]
fn zero_http_timeout_is_rejected_explicitly() {
    let result = PublicIpLocationResolver::new(
        settings(true),
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
    let endpoint = format!("http://{}/", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
        pending::<()>().await;
    });
    let resolver = resolver(true, vec![provider(ProviderKind::IpWhoIs, &endpoint)], 20);

    let result = resolver.resolve_ip_location(TEST_IP).await;

    assert!(matches!(result, Err(ClientInfoError::Provider(_))));
    server.abort();
}

fn resolver(enabled: bool, providers: Vec<ProviderEndpoint>, timeout_ms: u64) -> PublicIpLocationResolver {
    PublicIpLocationResolver::with_dependencies(
        settings(enabled),
        IpLocationClientConfig {
            request_timeout: Duration::from_millis(timeout_ms),
        },
        ResolverDependencies::new(providers, observer()),
    )
    .unwrap()
}

fn settings(enabled: bool) -> Arc<dyn IpLocationSettingsReader> {
    Arc::new(TestSettings {
        result: Ok(IpLocationConfig { enabled }),
    })
}

fn provider(kind: ProviderKind, endpoint: &str) -> ProviderEndpoint {
    ProviderEndpoint::new(kind, endpoint).unwrap()
}

async fn serve_once(status: &'static str, body: &'static str) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/", listener.local_addr().unwrap());
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0_u8; 2_048];
        let _ = socket.read(&mut request).await.unwrap();
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await.unwrap();
    });
    endpoint
}

fn assert_metric(rendered: &str, name_and_labels: &str, expected: u64) {
    let value = rendered
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{name_and_labels} ")))
        .map(|value| value.parse::<u64>().unwrap())
        .unwrap_or(0);
    assert_eq!(value, expected, "metric {name_and_labels} in:\n{rendered}");
}

fn observer() -> taco_tracing::InfrastructureObserver {
    let config = taco_tracing::parse_runtime_tracing_config(
        r#"{"log_level":"error","http":{"access_enabled":true,"capture_request_body":false,"capture_response_body":false,"capture_query_parameters":false,"capture_request_headers":false,"max_body_capture_bytes":0},"slow_operation_ms":{"postgres":500,"redis":100,"outbound_http":1000}}"#,
    )
    .unwrap();
    taco_tracing::InfrastructureObserver::new(taco_tracing::RuntimeTracingState::new(config))
}
