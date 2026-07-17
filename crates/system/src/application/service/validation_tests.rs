use super::*;

#[test]
fn page_validation_rejects_excessive_size_and_overflow() {
    let excessive = validate_page(CursorPageRequest {
        limit: kernel::pagination::MAX_CURSOR_LIMIT + 1,
        cursor: None,
    });
    let below_minimum = validate_page(CursorPageRequest { limit: 0, cursor: None });

    assert!(matches!(excessive, Err(SystemError::InvalidInput(message)) if message.key() == "errors.validation.cursor_limit_range"));
    assert!(matches!(below_minimum, Err(SystemError::InvalidInput(message)) if message.key() == "errors.validation.cursor_limit_range"));
}

#[test]
fn tracing_runtime_config_must_match_the_observability_schema() {
    let input = ConfigInput {
        config_key: constants::system_config::TRACING_CONFIG_KEY.into(),
        config_value: "{}".into(),
        ..valid_config_input()
    };

    let error = validate_runtime_config(&input).unwrap_err();

    assert!(matches!(error, SystemError::InvalidInput(message) if message.key() == "errors.system.invalid_observability_tracing_config"));
}

#[test]
fn sensitive_runtime_configs_cannot_be_public() {
    for key in [constants::system_config::CAPTCHA_CONFIG_KEY, constants::system_config::TRACING_CONFIG_KEY] {
        let error = reject_sensitive_public_config(key, true).unwrap_err();

        assert!(matches!(error, SystemError::Conflict(message) if message.key() == "errors.system.sensitive_config_private"));
    }
}

fn valid_config_input() -> ConfigInput {
    ConfigInput {
        config_name: "test".into(),
        config_key: "test.key".into(),
        config_value: "test".into(),
        config_type: "N".into(),
        public_read: false,
        remark: None,
    }
}
