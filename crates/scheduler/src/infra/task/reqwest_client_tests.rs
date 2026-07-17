use std::time::Duration;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
    time::sleep,
};

use crate::application::task::{HttpFailureCode, HttpTaskClient, OutboundHttpRequest};

use super::ReqwestHttpTaskClient;

const LARGE_BODY_BYTES: usize = 1_048_593;
const MAX_REQUEST_HEAD_BYTES: usize = 16_384;
const SERVER_DELAY: Duration = Duration::from_millis(10);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(2);
const SHORT_TIMEOUT: Duration = Duration::from_millis(20);
const TIMEOUT_SERVER_DELAY: Duration = Duration::from_millis(100);

#[tokio::test]
async fn captures_redirect_duplicate_headers_binary_and_large_body() {
    let expected_body = large_binary_body();
    let fixture = RedirectFixture::start(expected_body.clone()).await;
    let client = http_client(CLIENT_TIMEOUT);

    let response = client.send(request("GET", fixture.start_url.clone())).await.unwrap();
    fixture.server.await.unwrap();

    assert_eq!(response.head.status, 200);
    assert_eq!(response.head.final_url, fixture.final_url);
    assert_eq!(response.body, expected_body);
    assert!(response.duration >= SERVER_DELAY);
    let repeated = response
        .head
        .headers
        .iter()
        .filter(|header| header.name == "x-repeat")
        .map(|header| header.value.clone())
        .collect::<Vec<_>>();
    assert_eq!(repeated, vec![b"first".to_vec(), vec![128, 129]]);
}

#[tokio::test]
async fn classifies_invalid_method_and_connection_failure() {
    let client = http_client(CLIENT_TIMEOUT);
    let invalid = client.send(request("invalid method", "http://127.0.0.1/".into())).await.unwrap_err();
    assert_eq!(invalid.code, HttpFailureCode::RequestBuild);
    assert_eq!(invalid.response, None);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    drop(listener);
    let refused = client.send(request("GET", url)).await.unwrap_err();
    assert_eq!(refused.code, HttpFailureCode::Connect);
    assert_eq!(refused.response, None);
}

#[tokio::test]
async fn classifies_timeout_without_a_response() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        sleep(TIMEOUT_SERVER_DELAY).await;
        drop(stream);
    });

    let failure = http_client(SHORT_TIMEOUT).send(request("GET", url)).await.unwrap_err();
    server.await.unwrap();

    assert_eq!(failure.code, HttpFailureCode::Timeout);
    assert_eq!(failure.response, None);
    assert!(failure.duration >= SHORT_TIMEOUT);
}

#[tokio::test]
async fn body_read_failure_preserves_response_head() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\nX-Kept: value\r\nConnection: close\r\n\r\nshort".to_vec();
    let (url, server) = one_response_server(response).await;

    let failure = http_client(CLIENT_TIMEOUT).send(request("GET", url.clone())).await.unwrap_err();
    server.await.unwrap();

    assert_eq!(failure.code, HttpFailureCode::ResponseBody);
    let head = failure.response.expect("response body failure must retain the response head");
    assert_eq!(head.status, 200);
    assert_eq!(head.final_url, url);
    assert_eq!(head.headers.iter().find(|header| header.name == "x-kept").unwrap().value, b"value");
}

#[tokio::test]
async fn body_timeout_uses_timeout_code_without_a_response() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        read_request_head(&mut stream).await;
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\nshort")
            .await
            .unwrap();
        sleep(TIMEOUT_SERVER_DELAY).await;
    });

    let failure = http_client(SHORT_TIMEOUT).send(request("GET", url)).await.unwrap_err();
    server.await.unwrap();

    assert_eq!(failure.code, HttpFailureCode::Timeout);
    assert_eq!(failure.response, None);
    assert!(failure.duration >= SHORT_TIMEOUT);
}

struct RedirectFixture {
    start_url: String,
    final_url: String,
    server: JoinHandle<()>,
}

impl RedirectFixture {
    async fn start(body: Vec<u8>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let start_url = format!("http://{address}/start");
        let final_url = format!("http://{address}/final");
        let server = tokio::spawn(async move {
            let (first, _) = listener.accept().await.unwrap();
            respond(first, redirect_response()).await;
            let (second, _) = listener.accept().await.unwrap();
            sleep(SERVER_DELAY).await;
            respond(second, success_response(body)).await;
        });
        Self { start_url, final_url, server }
    }
}

async fn one_response_server(response: Vec<u8>) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        respond(stream, response).await;
    });
    (url, server)
}

async fn respond(mut stream: TcpStream, response: Vec<u8>) {
    read_request_head(&mut stream).await;
    stream.write_all(&response).await.unwrap();
    stream.shutdown().await.unwrap();
}

async fn read_request_head(stream: &mut TcpStream) {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        let read = stream.read(&mut chunk).await.unwrap();
        assert_ne!(read, 0, "client closed before sending a complete request head");
        request.extend_from_slice(&chunk[..read]);
        assert!(request.len() <= MAX_REQUEST_HEAD_BYTES, "test request head exceeded fixture limit");
    }
}

fn redirect_response() -> Vec<u8> {
    b"HTTP/1.1 302 Found\r\nLocation: /final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()
}

fn success_response(body: Vec<u8>) -> Vec<u8> {
    let mut response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nX-Repeat: first\r\nX-Repeat: ", body.len()).into_bytes();
    response.extend_from_slice(&[128, 129]);
    response.extend_from_slice(b"\r\nConnection: close\r\n\r\n");
    response.extend_from_slice(&body);
    response
}

fn large_binary_body() -> Vec<u8> {
    let mut body = vec![b'x'; LARGE_BODY_BYTES];
    body[..4].copy_from_slice(&[0, 159, 146, 150]);
    body
}

fn http_client(timeout: Duration) -> ReqwestHttpTaskClient {
    let client = reqwest::Client::builder().timeout(timeout).build().unwrap();
    ReqwestHttpTaskClient::new(client, observer())
}

fn observer() -> taco_tracing::InfrastructureObserver {
    let config = taco_tracing::parse_runtime_tracing_config(
        r#"{"log_level":"error","http":{"access_enabled":true,"capture_request_body":false,"capture_response_body":false,"capture_query_parameters":false,"capture_request_headers":false,"max_body_capture_bytes":0},"slow_operation_ms":{"postgres":500,"redis":100,"outbound_http":1000}}"#,
    )
    .unwrap();
    taco_tracing::InfrastructureObserver::new(taco_tracing::RuntimeTracingState::new(config))
}

fn request(method: &str, url: String) -> OutboundHttpRequest {
    OutboundHttpRequest {
        method: method.into(),
        url,
        headers: Vec::new(),
        body: None,
    }
}
