use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use axum::{Router, extract::State, http::StatusCode, routing::get};
use scheduler::{
    application::task::{SystemCacheRefreshPort, TaskExecutionContext, TaskExecutionFailure},
    infra::ReqwestHttpTaskClient,
};
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};

const HTTP_TIMEOUT: Duration = Duration::from_secs(2);

pub(super) struct HttpFixture {
    pub url: String,
    pub calls: Arc<AtomicUsize>,
    shutdown: oneshot::Sender<()>,
    server: JoinHandle<std::io::Result<()>>,
}

impl HttpFixture {
    pub async fn start() -> Self {
        let calls = Arc::new(AtomicUsize::new(0));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let router = Router::new().route("/task", get(record_http_call)).with_state(calls.clone());
        let (shutdown, receiver) = oneshot::channel();
        let server = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    receiver.await.expect("scheduler HTTP fixture shutdown sender was dropped");
                })
                .await
        });
        Self {
            url: format!("http://{address}/task"),
            calls,
            shutdown,
            server,
        }
    }

    pub async fn stop(self) {
        self.shutdown.send(()).expect("scheduler HTTP fixture stopped before shutdown");
        self.server
            .await
            .expect("scheduler HTTP fixture task panicked")
            .expect("scheduler HTTP fixture failed");
    }
}

pub(super) fn task_context() -> TaskExecutionContext {
    let client = reqwest::Client::builder().timeout(HTTP_TIMEOUT).build().unwrap();
    TaskExecutionContext {
        http_client: Arc::new(ReqwestHttpTaskClient::new(client)),
        system_cache: Arc::new(UnexpectedCachePort),
    }
}

async fn record_http_call(State(calls): State<Arc<AtomicUsize>>) -> StatusCode {
    calls.fetch_add(1, Ordering::SeqCst);
    StatusCode::NO_CONTENT
}

struct UnexpectedCachePort;

#[async_trait]
impl SystemCacheRefreshPort for UnexpectedCachePort {
    async fn refresh_config_cache(&self) -> Result<(), TaskExecutionFailure> {
        panic!("HTTP-only scheduler runtime test invoked config cache refresh")
    }

    async fn refresh_dict_cache(&self) -> Result<(), TaskExecutionFailure> {
        panic!("HTTP-only scheduler runtime test invoked dictionary cache refresh")
    }
}
