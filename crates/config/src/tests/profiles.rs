use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    path::PathBuf,
    sync::Mutex,
};

use super::*;

const CONFIG_EXAMPLE: &str = include_str!("../../../../config/config.example.yaml");
const EXPECTED_EXAMPLE_KEY_COUNT: usize = 71;
const COMMON_VARIABLES: [&str; 6] = [
    "TACO_DATABASE_PASSWORD",
    "TACO_REDIS_USERNAME",
    "TACO_REDIS_PASSWORD",
    "TACO_REDIS_DATABASE",
    "TACO_JWT_SECRET",
    "TACO_TURNSTILE_SECRET_KEY",
];
const DEPLOYMENT_VARIABLES: [&str; 8] = [
    "TACO_DATABASE_HOST",
    "TACO_DATABASE_PORT",
    "TACO_DATABASE_USERNAME",
    "TACO_DATABASE_NAME",
    "TACO_REDIS_HOST",
    "TACO_REDIS_PORT",
    "TACO_ADMIN_ORIGIN",
    "TACO_AVATAR_DIRECTORY",
];

struct ProfileEnvironment {
    values: BTreeMap<String, String>,
    reads: Mutex<BTreeSet<String>>,
}

impl ProfileEnvironment {
    fn valid() -> Self {
        let values = [
            ("TACO_DATABASE_HOST", "database.internal"),
            ("TACO_DATABASE_PORT", "6432"),
            ("TACO_DATABASE_USERNAME", "taco"),
            ("TACO_DATABASE_PASSWORD", "p@ss:%2F"),
            ("TACO_DATABASE_NAME", "taco"),
            ("TACO_REDIS_HOST", "redis.internal"),
            ("TACO_REDIS_PORT", "6380"),
            ("TACO_REDIS_USERNAME", ""),
            ("TACO_REDIS_PASSWORD", ""),
            ("TACO_REDIS_DATABASE", ""),
            ("TACO_JWT_SECRET", TEST_JWT_SECRET),
            ("TACO_TURNSTILE_SECRET_KEY", ""),
            ("TACO_ADMIN_ORIGIN", "https://admin.example.test"),
            ("TACO_AVATAR_DIRECTORY", "/var/lib/taco/avatars"),
        ];
        Self {
            values: values.into_iter().map(|(key, value)| (key.into(), value.into())).collect(),
            reads: Mutex::default(),
        }
    }

    fn reads(&self) -> BTreeSet<String> {
        self.reads.lock().unwrap().clone()
    }
}

impl EnvironmentReader for ProfileEnvironment {
    fn read(&self, variable: &str) -> Result<Option<String>, EnvironmentReadError> {
        self.reads.lock().unwrap().insert(variable.into());
        Ok(self.values.get(variable).cloned())
    }
}

#[test]
fn profiles_read_the_exact_shared_environment_contract() {
    let local_environment = ProfileEnvironment::valid();
    load_profile("config.local.yaml", &local_environment);
    assert_eq!(local_environment.reads(), expected_reads(&COMMON_VARIABLES));

    for file in ["config.dev.yaml", "config.prod.yaml", "config.example.yaml"] {
        let environment = ProfileEnvironment::valid();
        load_profile(file, &environment);
        let expected = COMMON_VARIABLES
            .into_iter()
            .chain(DEPLOYMENT_VARIABLES)
            .chain(crate::loader::FORBIDDEN_POSTGRES_ENVIRONMENT_VARIABLES)
            .map(str::to_owned)
            .collect();
        assert_eq!(environment.reads(), expected, "{file}");
    }
}

#[test]
fn local_and_remote_profiles_match_the_confirmed_differences() {
    let [local, dev, prod, example] = load_profiles();

    assert_profile(
        &local,
        &ProfileExpectation {
            host: "127.0.0.1",
            origin: "http://localhost:8082",
            redis_prefix: "taco:local",
            database_ssl_mode: DatabaseSslMode::Disable,
            redis_scheme: RedisScheme::Redis,
        },
    );
    assert_eq!(local.database.host, "localhost");
    assert_eq!(local.database.port, 5435);
    assert_eq!(local.uploads.avatar_directory, "storage/uploads/avatars");
    assert_eq!(whitelist_paths(&local), ["/health", "/metrics", "/openapi.json", "/docs"]);

    assert_profile(
        &dev,
        &ProfileExpectation {
            host: "0.0.0.0",
            origin: "https://admin.example.test",
            redis_prefix: "taco:dev",
            database_ssl_mode: DatabaseSslMode::VerifyFull,
            redis_scheme: RedisScheme::Rediss,
        },
    );
    assert_profile(
        &prod,
        &ProfileExpectation {
            host: "0.0.0.0",
            origin: "https://admin.example.test",
            redis_prefix: "taco:prod",
            database_ssl_mode: DatabaseSslMode::VerifyFull,
            redis_scheme: RedisScheme::Rediss,
        },
    );
    assert_eq!(dev.database.host, "database.internal");
    assert_eq!(dev.uploads.avatar_directory, "/var/lib/taco/avatars");
    assert_eq!(whitelist_paths(&dev), ["/health", "/metrics", "/openapi.json", "/docs"]);
    assert_eq!(whitelist_paths(&prod), ["/health", "/metrics"]);
    assert_eq!(example, prod);
}

#[test]
fn profiles_share_the_stable_runtime_policy_snapshot() {
    let [local, dev, prod, example] = load_profiles();

    for profile in [&dev, &prod, &example] {
        assert_stable_policies(&local, profile);
    }
}

#[test]
fn example_comments_every_mapping_key_and_structured_list_item() {
    let mut previous = "";
    let mut key_count = 0;

    for (index, line) in CONFIG_EXAMPLE.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            previous = "";
            continue;
        }
        if trimmed.starts_with('#') {
            previous = trimmed;
            continue;
        }
        key_count += 1;
        assert!(previous.starts_with('#'), "configuration key on line {} lacks an adjacent comment", index + 1);
        previous = trimmed;
    }

    assert_eq!(key_count, EXPECTED_EXAMPLE_KEY_COUNT);
}

fn load_profiles() -> [Settings; 4] {
    let environment = ProfileEnvironment::valid();
    [
        load_profile("config.local.yaml", &environment),
        load_profile("config.dev.yaml", &environment),
        load_profile("config.prod.yaml", &environment),
        load_profile("config.example.yaml", &environment),
    ]
}

fn load_profile(file: &str, environment: &dyn EnvironmentReader) -> Settings {
    let path = config_path(file);
    Settings::load_from_args_with_environment(
        [OsString::from("configuration-test"), OsString::from("--config"), path.into_os_string()],
        environment,
    )
    .unwrap()
}

fn config_path(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config").join(file)
}

fn expected_reads(values: &[&str]) -> BTreeSet<String> {
    values
        .iter()
        .copied()
        .chain(crate::loader::FORBIDDEN_POSTGRES_ENVIRONMENT_VARIABLES)
        .map(str::to_owned)
        .collect()
}

fn whitelist_paths(settings: &Settings) -> Vec<&str> {
    assert!(settings.auth.whitelist.iter().all(|rule| rule.methods == ["GET"]));
    settings.auth.whitelist.iter().map(|rule| rule.path_pattern.as_str()).collect()
}

struct ProfileExpectation<'a> {
    host: &'a str,
    origin: &'a str,
    redis_prefix: &'a str,
    database_ssl_mode: DatabaseSslMode,
    redis_scheme: RedisScheme,
}

fn assert_profile(settings: &Settings, expected: &ProfileExpectation<'_>) {
    assert_eq!(settings.server.host, expected.host);
    assert_eq!(settings.server.port, 3000);
    assert_eq!(settings.cors.allowed_origins, [expected.origin]);
    assert_eq!(settings.redis.key_prefix, expected.redis_prefix);
    assert_eq!(settings.database.ssl_mode, expected.database_ssl_mode);
    assert_eq!(settings.redis.scheme, expected.redis_scheme);
}

fn assert_stable_policies(left: &Settings, right: &Settings) {
    assert!(!left.database.auto_migrate && !right.database.auto_migrate);
    assert_eq!(left.server.port, right.server.port);
    assert_eq!(left.auth.refresh_cookie, right.auth.refresh_cookie);
    assert_eq!(left.user, right.user);
    assert_eq!(left.http, right.http);
    assert_eq!(left.metrics, right.metrics);
    assert_eq!(left.audit, right.audit);
    assert_eq!(left.client_info, right.client_info);
    assert_eq!(left.scheduler, right.scheduler);
    assert_eq!(left.database.scheme, right.database.scheme);
    assert_eq!(left.redis.protocol, right.redis.protocol);
}
