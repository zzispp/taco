use async_trait::async_trait;
use kernel::error::LocalizedError;

use crate::application::{AccountVerifier, AppError, AppResult};

use super::VALID_CAPTCHA_TOKEN;

pub(crate) struct TestCaptcha {
    enabled: bool,
}

impl TestCaptcha {
    pub(crate) fn enabled() -> Self {
        Self { enabled: true }
    }

    pub(super) fn disabled() -> Self {
        Self { enabled: false }
    }
}

#[async_trait]
impl AccountVerifier for TestCaptcha {
    async fn verify_account(&self, token: Option<&str>) -> AppResult<()> {
        if !self.enabled {
            return Ok(());
        }
        match token {
            Some(VALID_CAPTCHA_TOKEN) => Ok(()),
            Some(_) => Err(AppError::InvalidInput(LocalizedError::new("errors.captcha.verification_failed"))),
            None => Err(AppError::InvalidInput(LocalizedError::new("errors.captcha.verification_required"))),
        }
    }
}
