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
    let retired_prefix = ['h', 'o', 'o', 'k'].into_iter().collect::<String>();
    let secrets = [
        ["taco-local-", "development-jwt-", "secret-change-before-deploy"].concat(),
        format!("{retired_prefix}-local-development-jwt-secret-change-before-deploy"),
    ];

    for secret in secrets {
        let error = settings_with_jwt(JwtSettings { secret }).validate().unwrap_err();
        assert_eq!(error.to_string(), "jwt.secret must not use the known insecure development value");
    }
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
fn repository_config_example_has_the_full_interpolated_schema() {
    struct ExampleEnvironment;

    impl EnvironmentReader for ExampleEnvironment {
        fn read(&self, variable: &str) -> Result<Option<String>, EnvironmentReadError> {
            let value = match variable {
                "TACO_DATABASE_HOST" | "TACO_REDIS_HOST" => "localhost",
                "TACO_DATABASE_PORT" => "5435",
                "TACO_REDIS_PORT" => "6381",
                "TACO_DATABASE_USERNAME" | "TACO_DATABASE_NAME" => "postgres",
                "TACO_DATABASE_PASSWORD" => "unit-test-password",
                "TACO_JWT_SECRET" => TEST_JWT_SECRET,
                "TACO_TURNSTILE_SECRET_KEY" | "TACO_REDIS_USERNAME" | "TACO_REDIS_PASSWORD" | "TACO_REDIS_DATABASE" => "",
                "TACO_ADMIN_ORIGIN" => "https://admin.example.test",
                "TACO_AVATAR_DIRECTORY" => "storage/uploads/avatars",
                "TACO_LOG_DIRECTORY" => "logs",
                _ => return Ok(None),
            };
            Ok(Some(value.into()))
        }
    }

    let settings = crate::loader::deserialize_settings_with_environment(CONFIG_EXAMPLE, &ExampleEnvironment).unwrap();
    settings.validate().unwrap();

    assert_eq!(settings.jwt.secret, TEST_JWT_SECRET);
    assert_eq!(settings.captcha.cloudflare_turnstile.secret_key, "");
    assert_eq!(settings.database.password, "unit-test-password");
    assert_eq!(settings.redis.username, None);
    assert_eq!(settings.redis.password, None);
    assert_eq!(settings.refresh_cookie_config().unwrap().path, "/api/auth");
    assert_eq!(settings.cloudflare_turnstile_secret_key(), "");
}
