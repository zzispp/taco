use super::*;

#[test]
fn jwt_secret_trims_config_value() {
    let jwt = settings_with_jwt(JwtSettings {
        secret: format!("  {TEST_JWT_SECRET}  "),
    });

    assert_eq!(jwt.jwt_secret().unwrap(), TEST_JWT_SECRET);
}

#[test]
fn settings_validation_rejects_blank_jwt_secrets() {
    for secret in ["", "   "] {
        let settings = settings_with_jwt(JwtSettings { secret: secret.into() });

        assert_eq!(settings.validate().unwrap_err().to_string(), "jwt.secret cannot be blank");
    }
}

#[test]
fn settings_validation_rejects_known_insecure_jwt_secret() {
    let settings = settings_with_jwt(JwtSettings {
        secret: ["hook-local-", "development-jwt-", "secret-change-before-deploy"].concat(),
    });

    assert_eq!(
        settings.validate().unwrap_err().to_string(),
        "jwt.secret must not use the known insecure development value"
    );
}

#[test]
fn settings_validation_rejects_jwt_secrets_shorter_than_32_utf8_bytes() {
    let secrets = ["a".repeat(31), format!("{}a", "密".repeat(10))];

    for secret in secrets {
        let settings = settings_with_jwt(JwtSettings { secret });
        let error = settings.validate().unwrap_err();

        assert!(matches!(
            &error,
            SettingsError::JwtSecretTooShort {
                minimum_bytes: 32,
                actual_bytes: 31
            }
        ));
        assert_eq!(error.to_string(), "jwt.secret must be at least 32 UTF-8 bytes; got 31");
    }
}

#[test]
fn settings_validation_accepts_jwt_secrets_at_the_32_byte_boundary() {
    let secrets = [TEST_JWT_SECRET.to_owned(), format!("{}ab", "密".repeat(10))];

    for secret in secrets {
        assert_eq!(secret.len(), 32);
        settings_with_jwt(JwtSettings { secret }).validate().unwrap();
    }
}

#[test]
fn repository_config_example_has_the_full_schema_without_usable_credentials() {
    let settings = deserialize_settings(CONFIG_EXAMPLE).unwrap();

    assert_eq!(settings.jwt.secret, "");
    assert_eq!(settings.captcha.cloudflare_turnstile.secret_key, "");
    assert_eq!(settings.database.password, None);
    assert_eq!(settings.redis.username, None);
    assert_eq!(settings.redis.password, None);
    assert_eq!(settings.refresh_cookie_config().unwrap().path, "/api/auth");
    assert_eq!(settings.cloudflare_turnstile_secret_key(), "");
}
