use std::sync::Arc;

use crate::application::CaptchaUseCase;

#[derive(Clone)]
pub struct CaptchaApiState {
    pub captcha: Arc<dyn CaptchaUseCase>,
}

impl CaptchaApiState {
    pub fn new(captcha: Arc<dyn CaptchaUseCase>) -> Self {
        Self { captcha }
    }
}
