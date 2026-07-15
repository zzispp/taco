use super::*;

#[test]
fn turnstile_secret_trims_config_value() {
    let settings = settings_with_captcha(CaptchaSettings {
        cloudflare_turnstile: CloudflareTurnstileSettings {
            secret_key: "  turnstile-secret  ".into(),
        },
    });

    assert_eq!(settings.cloudflare_turnstile_secret_key(), "turnstile-secret");
}

#[test]
fn settings_validation_allows_blank_turnstile_secret_until_provider_use() {
    for secret_key in ["", "   "] {
        let settings = settings_with_captcha(CaptchaSettings {
            cloudflare_turnstile: CloudflareTurnstileSettings { secret_key: secret_key.into() },
        });

        settings.validate().unwrap();
        assert_eq!(settings.cloudflare_turnstile_secret_key(), "");
    }
}

#[test]
fn turnstile_secret_config_is_required_and_strict() {
    let source = minimal_config_without_auto_migrate();
    let missing = deserialize_settings(&source.replace(captcha_yaml(), ""));
    let unknown = deserialize_settings(&source.replace(captcha_yaml(), &format!("{captcha}    unexpected: true\n", captcha = captcha_yaml())));

    assert!(missing.is_err());
    assert!(unknown.is_err());
}
