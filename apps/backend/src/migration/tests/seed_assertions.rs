use sqlx::{PgPool, query_scalar};

const EXPECTED_ROLE_COUNT: i64 = 2;
const EXPECTED_MENU_COUNT: i64 = 44;
const EXPECTED_DEPT_COUNT: i64 = 10;
const EXPECTED_POST_COUNT: i64 = 4;
const EXPECTED_DICT_TYPE_COUNT: i64 = 5;
const EXPECTED_CONFIG_COUNT: i64 = 10;
const EXPECTED_PUBLIC_CONFIG_COUNT: i64 = 5;
const EXPECTED_CAPTCHA_DIFFICULTY: i64 = 4;
const EXPECTED_REFRESH_TOKEN_TTL_SECONDS: i64 = 604_800;
const EXPECTED_PASSWORD_MIN_LENGTH: i64 = 8;
const EXPECTED_AVATAR_MAX_BYTES: i64 = 2_097_152;
const EXPECTED_EXPORT_PAGE_SIZE: i64 = 100;
const EXPECTED_DASHBOARD_MENU_COUNT: i64 = 1;

pub(super) async fn assert_seed_data_exists(pool: &PgPool) {
    assert_eq!(table_count(pool, "sys_role").await, EXPECTED_ROLE_COUNT);
    assert_eq!(table_count(pool, "sys_menu").await, EXPECTED_MENU_COUNT);
    assert_eq!(table_count(pool, "sys_dept").await, EXPECTED_DEPT_COUNT);
    assert_eq!(table_count(pool, "sys_post").await, EXPECTED_POST_COUNT);
    assert_eq!(table_count(pool, "sys_dict_type").await, EXPECTED_DICT_TYPE_COUNT);
    assert_eq!(table_count(pool, "sys_config").await, EXPECTED_CONFIG_COUNT);
    assert_eq!(public_config_count(pool).await, EXPECTED_PUBLIC_CONFIG_COUNT);
    assert_seed_config_values(pool).await;
    assert_seed_config_remarks(pool).await;
    assert_dashboard_menu_exists(pool).await;
}

async fn assert_seed_config_values(pool: &PgPool) {
    let captcha = captcha_config(pool).await;
    assert_eq!(captcha["provider"], "cap");
    assert_eq!(captcha["providers"]["cap"]["challenge_difficulty"], EXPECTED_CAPTCHA_DIFFICULTY);
    assert_eq!(captcha["providers"]["cloudflare_turnstile"]["site_key"], "");
    assert_eq!(captcha["providers"]["cloudflare_turnstile"]["secret_key"], "");
    assert_eq!(token_config(pool).await["refresh_token_ttl_seconds"], EXPECTED_REFRESH_TOKEN_TTL_SECONDS);
    assert_eq!(password_policy(pool).await["min_length"], EXPECTED_PASSWORD_MIN_LENGTH);
    assert_eq!(avatar_config(pool).await["max_bytes"], EXPECTED_AVATAR_MAX_BYTES);
    assert_eq!(export_batch_config(pool).await["page_size"], EXPECTED_EXPORT_PAGE_SIZE);
    assert_eq!(site_display_config(pool).await["site_name"], "taco");
    assert_eq!(initial_password(pool).await, "12345678");
    assert_eq!(mode_theme(pool).await, "theme-light");
    assert_legacy_captcha_configs_removed(pool).await;
}

async fn assert_seed_config_remarks(pool: &PgPool) {
    assert_eq!(non_empty_config_remark_count(pool).await, EXPECTED_CONFIG_COUNT);
    assert_config_remark_contains(
        pool,
        "sys.user.passwordPolicy",
        &[
            "min_length",
            "max_length",
            "require_letter",
            "require_number",
            "require_symbol",
            "forbid_username_contains",
        ],
    )
    .await;
    assert_config_remark_contains(
        pool,
        "sys.account.captchaConfig",
        &["enabled", "provider", "providers.cap", "providers.cloudflare_turnstile"],
    )
    .await;
    assert_config_remark_contains(pool, "sys.auth.tokenConfig", &["access_token_ttl_seconds", "refresh_token_ttl_seconds"]).await;
    assert_config_remark_contains(pool, "sys.upload.avatarConfig", &["max_bytes"]).await;
    assert_config_remark_contains(pool, "sys.export.batchConfig", &["page_size"]).await;
    assert_config_remark_contains(pool, "sys.site.displayConfig", &["site_name", "logo_url", "footer_text"]).await;
}

async fn initial_password(pool: &PgPool) -> String {
    query_scalar::<_, String>("SELECT config_value FROM sys_config WHERE config_key = 'sys.user.initPassword'")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn mode_theme(pool: &PgPool) -> String {
    query_scalar::<_, String>("SELECT config_value FROM sys_config WHERE config_key = 'sys.index.modeTheme'")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn public_config_count(pool: &PgPool) -> i64 {
    query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_config WHERE public_read = TRUE")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn assert_legacy_captcha_configs_removed(pool: &PgPool) {
    let legacy_count: i64 = query_scalar("SELECT COUNT(*) FROM sys_config WHERE config_key = ANY($1)")
        .bind([
            "sys.account.captchaEnabled",
            "sys.account.captchaProvider",
            "sys.account.captchaPublicConfig",
            "sys.account.captchaPrivateConfig",
        ])
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(legacy_count, 0);
}

async fn assert_dashboard_menu_exists(pool: &PgPool) {
    let count: i64 =
        query_scalar("SELECT COUNT(*) FROM sys_menu WHERE path = '/dashboard' AND perms = 'system:dashboard:view' AND visible = '0' AND status = '0'")
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(count, EXPECTED_DASHBOARD_MENU_COUNT);
}

async fn captcha_config(pool: &PgPool) -> serde_json::Value {
    config_json(pool, "sys.account.captchaConfig").await
}

async fn token_config(pool: &PgPool) -> serde_json::Value {
    config_json(pool, "sys.auth.tokenConfig").await
}

async fn password_policy(pool: &PgPool) -> serde_json::Value {
    config_json(pool, "sys.user.passwordPolicy").await
}

async fn avatar_config(pool: &PgPool) -> serde_json::Value {
    config_json(pool, "sys.upload.avatarConfig").await
}

async fn export_batch_config(pool: &PgPool) -> serde_json::Value {
    config_json(pool, "sys.export.batchConfig").await
}

async fn site_display_config(pool: &PgPool) -> serde_json::Value {
    config_json(pool, "sys.site.displayConfig").await
}

async fn config_json(pool: &PgPool, key: &str) -> serde_json::Value {
    let value: String = query_scalar("SELECT config_value FROM sys_config WHERE config_key = $1")
        .bind(key)
        .fetch_one(pool)
        .await
        .unwrap();
    serde_json::from_str(&value).unwrap()
}

async fn non_empty_config_remark_count(pool: &PgPool) -> i64 {
    query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_config WHERE NULLIF(trim(remark), '') IS NOT NULL")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn assert_config_remark_contains(pool: &PgPool, key: &str, fields: &[&str]) {
    let remark = config_remark(pool, key).await;
    for field in fields {
        assert!(remark.contains(field), "{key} remark should contain {field}: {remark}");
    }
}

async fn config_remark(pool: &PgPool, key: &str) -> String {
    query_scalar("SELECT COALESCE(remark, '') FROM sys_config WHERE config_key = $1")
        .bind(key)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn table_count(pool: &PgPool, table: &str) -> i64 {
    let sql = match table {
        "sys_role" => "SELECT COUNT(*) FROM sys_role",
        "sys_menu" => "SELECT COUNT(*) FROM sys_menu",
        "sys_dept" => "SELECT COUNT(*) FROM sys_dept",
        "sys_post" => "SELECT COUNT(*) FROM sys_post",
        "sys_dict_type" => "SELECT COUNT(*) FROM sys_dict_type",
        "sys_config" => "SELECT COUNT(*) FROM sys_config",
        _ => panic!("unexpected table: {table}"),
    };
    query_scalar::<_, i64>(sql).fetch_one(pool).await.unwrap()
}
