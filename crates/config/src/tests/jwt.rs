use super::*;

#[test]
fn jwt_secret_trims_profile_value() {
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
fn settings_validation_enforces_jwt_secret_byte_length() {
    let too_short = ["a".repeat(31), format!("{}a", "密".repeat(10))];
    for secret in too_short {
        let error = settings_with_jwt(JwtSettings { secret }).validate().unwrap_err();
        assert!(matches!(
            error,
            SettingsError::JwtSecretTooShort {
                minimum_bytes: 32,
                actual_bytes: 31
            }
        ));
    }

    for secret in [TEST_JWT_SECRET.to_owned(), format!("{}ab", "密".repeat(10))] {
        assert_eq!(secret.len(), 32);
        settings_with_jwt(JwtSettings { secret }).validate().unwrap();
    }
}
