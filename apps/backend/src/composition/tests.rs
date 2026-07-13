use configuration::{
    AuthSettings, CorsSettings, DatabaseSettings, HttpSettings, JwtSettings, MetricsSettings, RedisSettings, SchedulerHttpClientSettings,
    SchedulerRuntimeSettings, SchedulerSettings, ServerSettings, Settings, TracingFileSettings, TracingSettings, UploadSettings,
};
use rbac::application::PermissionRequirement;

use super::routes::{auth_whitelist, ensure_auth_whitelist_rule, route_permissions};

const TEST_SERVER_PORT: u16 = 3000;
const TEST_DATABASE_PORT: u16 = 5432;
const TEST_REDIS_PORT: u16 = 6379;
const TEST_HTTP_TIMEOUT_MS: u64 = 30_000;
const TEST_SCHEDULER_REQUEST_TIMEOUT_MS: u64 = 30_000;
const TEST_SCHEDULER_RECONCILE_INTERVAL_MS: u64 = 1_000;
const TEST_REDIS_DATABASE: u16 = 0;

#[test]
fn ensure_auth_whitelist_rule_adds_rule_once() {
    let mut rules = vec![];

    ensure_auth_whitelist_rule(&mut rules, &["GET"], "/api/auth/me");
    ensure_auth_whitelist_rule(&mut rules, &["GET"], "/api/auth/me");

    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].methods, vec!["GET"]);
    assert_eq!(rules[0].path_pattern, "/api/auth/me");
}

#[test]
fn auth_whitelist_includes_public_avatar_files() {
    let rules = auth_whitelist(&test_settings());

    let avatar_rule = rules.iter().find(|rule| rule.path_pattern == "/uploads/avatars/{*file}");

    assert_eq!(avatar_rule.map(|rule| rule.methods.clone()), Some(vec!["GET".to_owned()]));
}

#[test]
fn route_permissions_use_the_handler_permission_contract() {
    let rules = route_permissions();
    let list_jobs = rules.iter().find(|rule| rule.handler == "list_jobs").unwrap();
    let cron_next_times = rules.iter().find(|rule| rule.handler == "cron_next_times").unwrap();
    let job_log_detail = rules.iter().find(|rule| rule.handler == "get_job_log_detail").unwrap();

    assert_eq!(list_jobs.requirement, PermissionRequirement::all_of(&["system:job:list"]));
    assert_eq!(
        cron_next_times.requirement,
        PermissionRequirement::any_of(&["system:job:import", "system:job:edit"])
    );
    assert_eq!(
        job_log_detail.requirement,
        PermissionRequirement::all_of(&["system:job:log:query", "system:job:log:detail"])
    );
    assert_eq!(job_log_detail.path_pattern, "/api/system/job-logs/{id}/detail");
    assert!(!job_log_detail.requirement.is_satisfied_by(&["system:job:log:query".into()]));
    assert!(!job_log_detail.requirement.is_satisfied_by(&["system:job:log:detail".into()]));
    assert!(
        job_log_detail
            .requirement
            .is_satisfied_by(&["system:job:log:query".into(), "system:job:log:detail".into()])
    );
    let registered = rbac::inventory::iter::<rbac::application::ProtectedHandler>
        .into_iter()
        .find(|handler| handler.function == "get_job_log_detail")
        .unwrap();
    assert_eq!(registered.requirement, job_log_detail.requirement);
}

#[test]
fn notice_routes_use_the_complete_permission_contract() {
    let rules = route_permissions();
    let expected = [
        (
            "list_notices",
            vec!["GET"],
            "/api/system/notices",
            PermissionRequirement::all_of(&["system:notice:list"]),
        ),
        (
            "create_notice",
            vec!["POST"],
            "/api/system/notices",
            PermissionRequirement::all_of(&["system:notice:add"]),
        ),
        (
            "replace_notice",
            vec!["PUT"],
            "/api/system/notices/{id}",
            PermissionRequirement::all_of(&["system:notice:edit"]),
        ),
        (
            "delete_notice",
            vec!["DELETE"],
            "/api/system/notices/{id}",
            PermissionRequirement::all_of(&["system:notice:remove"]),
        ),
        (
            "delete_notices",
            vec!["DELETE"],
            "/api/system/notices/batch",
            PermissionRequirement::all_of(&["system:notice:remove"]),
        ),
        (
            "list_notice_readers",
            vec!["GET"],
            "/api/system/notices/{id}/readers",
            PermissionRequirement::all_of(&["system:notice:list"]),
        ),
    ];

    for (handler, methods, path, requirement) in expected {
        let rule = rules.iter().find(|rule| rule.handler == handler).unwrap();
        assert_eq!(rule.methods, methods, "unexpected methods for {handler}");
        assert_eq!(rule.path_pattern, path, "unexpected path for {handler}");
        assert_eq!(rule.requirement, requirement, "unexpected permission for {handler}");
    }
    assert!(rules.iter().all(|rule| rule.handler != "get_notice"));
}

fn test_settings() -> Settings {
    Settings {
        server: test_server_settings(),
        database: test_database_settings(),
        jwt: JwtSettings { secret: "secret".into() },
        auth: AuthSettings { whitelist: vec![] },
        cors: test_cors_settings(),
        http: test_http_settings(),
        metrics: MetricsSettings { enabled: true },
        redis: test_redis_settings(),
        scheduler: test_scheduler_settings(),
        uploads: UploadSettings::default(),
        tracing: test_tracing_settings(),
    }
}

fn test_scheduler_settings() -> SchedulerSettings {
    SchedulerSettings {
        http_client: SchedulerHttpClientSettings {
            request_timeout_ms: TEST_SCHEDULER_REQUEST_TIMEOUT_MS,
        },
        runtime: SchedulerRuntimeSettings {
            reconcile_interval_ms: TEST_SCHEDULER_RECONCILE_INTERVAL_MS,
        },
    }
}

fn test_server_settings() -> ServerSettings {
    ServerSettings {
        host: "127.0.0.1".into(),
        port: TEST_SERVER_PORT,
    }
}

fn test_database_settings() -> DatabaseSettings {
    DatabaseSettings {
        auto_migrate: false,
        url: None,
        scheme: "postgres".into(),
        host: "localhost".into(),
        port: TEST_DATABASE_PORT,
        username: "postgres".into(),
        password: Some("postgres".into()),
        name: "postgres".into(),
    }
}

fn test_cors_settings() -> CorsSettings {
    CorsSettings {
        allowed_origins: vec!["*".into()],
        allowed_methods: vec!["*".into()],
        allowed_headers: vec!["*".into()],
        exposed_headers: vec!["*".into()],
        allow_credentials: false,
        max_age_seconds: None,
    }
}

fn test_http_settings() -> HttpSettings {
    HttpSettings {
        request_timeout_ms: TEST_HTTP_TIMEOUT_MS,
        compression_enabled: true,
    }
}

fn test_redis_settings() -> RedisSettings {
    RedisSettings {
        url: None,
        scheme: "redis".into(),
        host: "localhost".into(),
        port: TEST_REDIS_PORT,
        username: None,
        password: None,
        database: Some(TEST_REDIS_DATABASE),
        protocol: Some("resp3".into()),
        key_prefix: "taco".into(),
    }
}

fn test_tracing_settings() -> TracingSettings {
    TracingSettings {
        log_level: "info".into(),
        file: TracingFileSettings {
            enabled: false,
            directory: "logs".into(),
            prefix: "taco.log".into(),
        },
    }
}
