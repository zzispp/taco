use std::time::Instant;

use async_trait::async_trait;
use reqwest::{Method, RequestBuilder, Response};
use taco_tracing::{InfrastructureDependency, InfrastructureObserver};

use crate::application::task::{
    HttpFailureCode, HttpTaskClient, OutboundHttpFailure, OutboundHttpHeader, OutboundHttpRequest, OutboundHttpResponse, OutboundHttpResponseHead,
};

#[derive(Clone)]
pub struct ReqwestHttpTaskClient {
    client: reqwest::Client,
    observer: InfrastructureObserver,
}

impl ReqwestHttpTaskClient {
    pub fn new(client: reqwest::Client, observer: InfrastructureObserver) -> Self {
        Self { client, observer }
    }
}

#[async_trait]
impl HttpTaskClient for ReqwestHttpTaskClient {
    async fn send(&self, request: OutboundHttpRequest) -> Result<OutboundHttpResponse, OutboundHttpFailure> {
        let started = Instant::now();
        let result = send_request(&self.client, request, started).await;
        self.observer.record(
            InfrastructureDependency::OutboundHttp,
            "scheduler_http_request",
            started.elapsed(),
            result.is_ok(),
        );
        result
    }
}

async fn send_request(client: &reqwest::Client, request: OutboundHttpRequest, started: Instant) -> Result<OutboundHttpResponse, OutboundHttpFailure> {
    let method = parse_method(&request.method, started)?;
    let builder = build_request(client, method, request);
    let response = builder.send().await.map_err(|error| request_failure(error, started))?;
    read_response(response, started).await
}

fn parse_method(method: &str, started: Instant) -> Result<Method, OutboundHttpFailure> {
    Method::from_bytes(method.as_bytes()).map_err(|_| OutboundHttpFailure {
        code: HttpFailureCode::RequestBuild,
        duration: started.elapsed(),
        response: None,
    })
}

fn build_request(client: &reqwest::Client, method: Method, request: OutboundHttpRequest) -> RequestBuilder {
    let mut builder = client.request(method, request.url);
    for (name, value) in request.headers {
        builder = builder.header(name, value);
    }
    if let Some(body) = request.body {
        builder = builder.body(body);
    }
    builder
}

async fn read_response(response: Response, started: Instant) -> Result<OutboundHttpResponse, OutboundHttpFailure> {
    let head = response_head(&response);
    match response.bytes().await {
        Ok(body) => Ok(OutboundHttpResponse {
            head,
            body: body.to_vec(),
            duration: started.elapsed(),
        }),
        Err(error) => Err(response_body_failure(error, started, head)),
    }
}

fn response_head(response: &Response) -> OutboundHttpResponseHead {
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| OutboundHttpHeader {
            name: name.as_str().to_owned(),
            value: value.as_bytes().to_vec(),
        })
        .collect();
    OutboundHttpResponseHead {
        status: response.status().as_u16(),
        headers,
        final_url: response.url().to_string(),
    }
}

fn request_failure(error: reqwest::Error, started: Instant) -> OutboundHttpFailure {
    let code = classify_request_failure(&error);
    OutboundHttpFailure {
        code,
        duration: started.elapsed(),
        response: None,
    }
}

fn response_body_failure(error: reqwest::Error, started: Instant, response: OutboundHttpResponseHead) -> OutboundHttpFailure {
    let timeout = error.is_timeout();
    if timeout {
        return OutboundHttpFailure {
            code: HttpFailureCode::Timeout,
            duration: started.elapsed(),
            response: None,
        };
    }
    OutboundHttpFailure {
        code: HttpFailureCode::ResponseBody,
        duration: started.elapsed(),
        response: Some(response),
    }
}

fn classify_request_failure(error: &reqwest::Error) -> HttpFailureCode {
    if error.is_timeout() {
        return HttpFailureCode::Timeout;
    }
    if error.is_connect() {
        return HttpFailureCode::Connect;
    }
    if error.is_builder() {
        return HttpFailureCode::RequestBuild;
    }
    HttpFailureCode::Request
}
