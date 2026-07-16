use std::{collections::BTreeMap, sync::Mutex};

use super::*;

#[derive(Default)]
struct MapEnvironment {
    values: BTreeMap<String, String>,
    reads: Mutex<Vec<String>>,
}

impl MapEnvironment {
    fn with(values: &[(&str, &str)]) -> Self {
        Self {
            values: values.iter().map(|(key, value)| ((*key).into(), (*value).into())).collect(),
            reads: Mutex::default(),
        }
    }

    fn reads(&self) -> Vec<String> {
        self.reads.lock().unwrap().clone()
    }
}

impl EnvironmentReader for MapEnvironment {
    fn read(&self, variable: &str) -> Result<Option<String>, EnvironmentReadError> {
        self.reads.lock().unwrap().push(variable.into());
        Ok(self.values.get(variable).cloned())
    }
}

#[test]
fn interpolation_parses_each_target_type_without_yaml_reparsing() {
    let source = minimal_config()
        .replace("port: 5435", "port: \"${DATABASE_PORT}\"")
        .replace("password: \"unit-test-password\"", "password: \"${DATABASE_PASSWORD}\"")
        .replace("metrics:\n  enabled: true", "metrics:\n  enabled: ${METRICS_ENABLED}");
    let environment = MapEnvironment::with(&[
        ("DATABASE_PORT", "6543"),
        ("DATABASE_PASSWORD", "p:a#ss$\\\"'\n[]{}"),
        ("METRICS_ENABLED", "false"),
    ]);

    let settings = crate::loader::deserialize_settings_with_environment(&source, &environment).unwrap();

    assert_eq!(settings.database.port, 6543);
    assert_eq!(settings.database.password, "p:a#ss$\\\"'\n[]{}");
    assert!(!settings.metrics.enabled);
}

#[test]
fn interpolation_reports_missing_variable_at_the_field_path() {
    let source = minimal_config().replace("port: 5435", "port: \"${DATABASE_PORT}\"");

    let error = crate::loader::deserialize_settings_with_environment(&source, &MapEnvironment::default()).unwrap_err();

    assert!(matches!(
        error,
        SettingsError::MissingEnvironmentVariable { ref variable, ref path }
            if variable == "DATABASE_PORT" && path == "database.port"
    ));
}

#[test]
fn interpolation_type_errors_do_not_expose_environment_values() {
    let source = minimal_config().replace("port: 5435", "port: \"${DATABASE_PORT}\"");
    let environment = MapEnvironment::with(&[("DATABASE_PORT", "sensitive-invalid-port")]);

    let error = crate::loader::deserialize_settings_with_environment(&source, &environment).unwrap_err();

    assert!(matches!(
        error,
        SettingsError::InvalidEnvironmentValue { ref variable, ref path, expected: "u16" }
            if variable == "DATABASE_PORT" && path == "database.port"
    ));
    assert!(!error.to_string().contains("sensitive-invalid-port"));
}

#[test]
fn interpolation_rejects_embedded_and_default_expressions() {
    for invalid in [
        "prefix-${DATABASE_HOST}",
        "${DATABASE_HOST:-localhost}",
        "${1DATABASE_HOST}",
        "${DATABASE.HOST}",
        "${}",
    ] {
        let source = minimal_config().replacen("host: \"localhost\"", &format!("host: \"{invalid}\""), 1);
        let error = crate::loader::deserialize_settings_with_environment(&source, &MapEnvironment::default()).unwrap_err();

        assert!(matches!(
            error,
            SettingsError::InvalidEnvironmentPlaceholder { ref path } if path == "database.host"
        ));
    }
}

#[test]
fn interpolation_rejects_yaml_collection_injection() {
    let source = minimal_config().replace("allowed_origins: [\"https://admin.example.test\"]", "allowed_origins: \"${CORS_ORIGINS}\"");
    let environment = MapEnvironment::with(&[("CORS_ORIGINS", "[https://evil.example.test]")]);

    let error = crate::loader::deserialize_settings_with_environment(&source, &environment).unwrap_err();

    assert!(matches!(
        error,
        SettingsError::InvalidEnvironmentValue { ref variable, ref path, expected: "sequence" }
            if variable == "CORS_ORIGINS" && path == "cors.allowed_origins"
    ));
    assert!(!error.to_string().contains("evil.example.test"));
}

#[test]
fn interpolation_uses_strict_target_parsing_and_nested_paths() {
    let bool_source = minimal_config().replace("metrics:\n  enabled: true", "metrics:\n  enabled: \"${METRICS_ENABLED}\"");
    let bool_environment = MapEnvironment::with(&[("METRICS_ENABLED", "yes")]);
    let bool_error = crate::loader::deserialize_settings_with_environment(&bool_source, &bool_environment).unwrap_err();

    assert!(
        matches!(
            &bool_error,
            SettingsError::InvalidEnvironmentValue { variable, path, expected: "bool" }
                if variable == "METRICS_ENABLED" && path == "metrics.enabled"
        ),
        "{bool_error:?}"
    );

    let nested_source = minimal_config().replace(
        "whitelist: []",
        "whitelist:\n    - methods: [\"${AUTH_METHOD}\"]\n      path_pattern: \"/health\"",
    );
    let nested_error = crate::loader::deserialize_settings_with_environment(&nested_source, &MapEnvironment::default()).unwrap_err();

    assert!(matches!(
        nested_error,
        SettingsError::MissingEnvironmentVariable { ref variable, ref path }
            if variable == "AUTH_METHOD" && path == "auth.whitelist[0].methods[0]"
    ));
}

#[test]
fn interpolation_expands_each_placeholder_once() {
    let source = minimal_config().replace("password: \"unit-test-password\"", "password: \"${DATABASE_PASSWORD}\"");
    let environment = MapEnvironment::with(&[("DATABASE_PASSWORD", "${SECOND_SECRET}"), ("SECOND_SECRET", "must-not-be-read")]);

    let settings = crate::loader::deserialize_settings_with_environment(&source, &environment).unwrap();

    assert_eq!(settings.database.password, "${SECOND_SECRET}");
    assert_eq!(environment.reads(), vec!["DATABASE_PASSWORD"]);
}

#[test]
fn empty_environment_values_map_to_none_only_for_optional_fields() {
    let source = minimal_config()
        .replace("password: \"unit-test-password\"", "password: \"${DATABASE_PASSWORD}\"")
        .replace("username: \"default\"", "username: \"${REDIS_USERNAME}\"")
        .replace("password: \"\"", "password: \"${REDIS_PASSWORD}\"")
        .replace("database:\n  protocol", "database: \"${REDIS_DATABASE}\"\n  protocol");
    let environment = MapEnvironment::with(&[
        ("DATABASE_PASSWORD", ""),
        ("REDIS_USERNAME", ""),
        ("REDIS_PASSWORD", ""),
        ("REDIS_DATABASE", ""),
    ]);

    let settings = crate::loader::deserialize_settings_with_environment(&source, &environment).unwrap();

    assert_eq!(settings.database.password, "");
    assert_eq!(settings.redis.username, None);
    assert_eq!(settings.redis.password, None);
    assert_eq!(settings.redis.database, None);
}

#[test]
fn interpolation_reports_non_unicode_values_without_exposing_them() {
    struct NonUnicodeEnvironment;

    impl EnvironmentReader for NonUnicodeEnvironment {
        fn read(&self, _: &str) -> Result<Option<String>, EnvironmentReadError> {
            Err(EnvironmentReadError::NotUnicode)
        }
    }

    let source = minimal_config().replace("port: 5435", "port: \"${DATABASE_PORT}\"");
    let error = crate::loader::deserialize_settings_with_environment(&source, &NonUnicodeEnvironment).unwrap_err();

    assert!(matches!(
        error,
        SettingsError::InvalidEnvironmentEncoding { ref variable, ref path }
            if variable == "DATABASE_PORT" && path == "database.port"
    ));
}
