mod app_state;
mod auth;
mod commands;
mod composition;
mod docs;
mod migration;
mod openapi;
mod http_config;
mod startup;
mod system;

#[cfg(test)]
mod app_tests;

type BackendResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> BackendResult<()> {
    commands::run().await
}
