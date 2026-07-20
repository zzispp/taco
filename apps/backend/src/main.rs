mod app_state;
mod auth;
mod commands;
mod composition;
mod docs;
pub mod embedded_frontend;
mod http_config;
mod installation_mode;
mod migration;
mod openapi;
mod startup;
mod system;

#[cfg(test)]
mod app_tests;

type BackendResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> BackendResult<()> {
    commands::run().await
}
