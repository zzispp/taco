use super::*;

#[test]
fn audit_outbox_config_rejects_each_zero_value() {
    let mut workers = settings_with_database(database_parts());
    workers.audit.outbox.worker_count = 0;
    let mut lease = settings_with_database(database_parts());
    lease.audit.outbox.lease_duration_ms = 0;
    let mut retention = settings_with_database(database_parts());
    retention.audit.outbox.processed_retention_days = 0;

    assert!(matches!(
        workers.audit_config(),
        Err(SettingsError::NonPositiveNumber("audit.outbox.worker_count"))
    ));
    assert!(matches!(
        lease.audit_config(),
        Err(SettingsError::NonPositiveNumber("audit.outbox.lease_duration_ms"))
    ));
    assert!(matches!(
        retention.audit_config(),
        Err(SettingsError::NonPositiveNumber("audit.outbox.processed_retention_days"))
    ));
}

#[test]
fn client_info_http_timeout_must_be_positive() {
    let mut settings = settings_with_database(database_parts());
    settings.client_info.ip_location.request_timeout_ms = 0;

    assert!(matches!(
        settings.client_info_config(),
        Err(SettingsError::NonPositiveNumber("client_info.ip_location.request_timeout_ms"))
    ));
}

#[test]
fn refresh_cookie_config_normalizes_domain_and_path() {
    let mut settings = settings_with_database(database_parts());
    settings.auth.refresh_cookie = RefreshCookieSettings {
        secure: true,
        domain: Some("  admin.example.test  ".into()),
        path: "  /api/auth  ".into(),
    };

    assert_eq!(
        settings.refresh_cookie_config().unwrap(),
        RefreshCookieSettings {
            secure: true,
            domain: Some("admin.example.test".into()),
            path: "/api/auth".into(),
        }
    );
}

#[test]
fn refresh_cookie_config_rejects_blank_domain_and_relative_path() {
    let mut blank_domain = settings_with_database(database_parts());
    blank_domain.auth.refresh_cookie.domain = Some("   ".into());
    let mut relative_path = settings_with_database(database_parts());
    relative_path.auth.refresh_cookie.path = "api/auth".into();

    assert!(matches!(
        blank_domain.refresh_cookie_config(),
        Err(SettingsError::BlankConfigValue("auth.refresh_cookie.domain"))
    ));
    assert!(matches!(
        relative_path.refresh_cookie_config(),
        Err(SettingsError::InvalidCookiePath("auth.refresh_cookie.path"))
    ));
}

#[test]
fn refresh_cookie_config_rejects_insecure_transport() {
    let mut settings = settings_with_database(database_parts());
    settings.auth.refresh_cookie.secure = false;

    assert!(matches!(settings.refresh_cookie_config(), Err(SettingsError::InsecureRefreshCookie)));
}
