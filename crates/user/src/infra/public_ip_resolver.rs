use async_trait::async_trait;

use crate::application::{AppError, AppResult, PublicIpResolver};

#[derive(Clone, Copy, Default)]
pub struct PublicIpAddressResolver;

#[async_trait]
impl PublicIpResolver for PublicIpAddressResolver {
    async fn resolve_public_ip(&self) -> AppResult<String> {
        public_ip_address::perform_lookup(None)
            .await
            .map(|response| response.ip.to_string())
            .map_err(public_ip_error)
    }
}

fn public_ip_error(error: public_ip_address::error::Error) -> AppError {
    AppError::Infrastructure(format!("public ip lookup error: {error:?}"))
}
