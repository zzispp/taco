use constants::system_config::CAPTCHA_CONFIG_KEY;
use kernel::error::LocalizedError;
use serde_json::Value;

use super::{CaptchaError, CaptchaResult};

pub fn parse_captcha_config_json(value: &str) -> CaptchaResult<Value> {
    serde_json::from_str(value).map_err(|error| {
        hook_tracing::error_with_fields!("invalid captcha runtime config JSON", &error, key = CAPTCHA_CONFIG_KEY);
        invalid_captcha_config_json()
    })
}

pub fn invalid_captcha_config_json() -> CaptchaError {
    CaptchaError::InvalidInput(LocalizedError::new("errors.captcha.invalid_config_json").with_param("key", CAPTCHA_CONFIG_KEY))
}

#[cfg(test)]
mod tests {
    use super::parse_captcha_config_json;
    use crate::application::CaptchaError;

    #[test]
    fn captcha_json_parser_owns_its_stable_error() {
        let CaptchaError::InvalidInput(error) = parse_captcha_config_json("invalid").unwrap_err() else {
            panic!("invalid JSON must be an input error");
        };
        assert_eq!(error.key(), "errors.captcha.invalid_config_json");
        assert_eq!(error.params()[0].value(), constants::system_config::CAPTCHA_CONFIG_KEY);
    }
}
