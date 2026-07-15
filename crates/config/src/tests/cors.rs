use super::*;

#[test]
fn validated_cors_accepts_a_concrete_http_origin() {
    let cors = settings_with_cors(cors_settings()).validated_cors().unwrap();

    assert_eq!(cors.allowed_origins, ValidatedCorsList::Values(vec!["https://admin.example.test".into()]));
    assert_eq!(cors.allowed_methods, ValidatedCorsList::Any);
    assert_eq!(cors.allowed_headers, ValidatedCorsList::Any);
    assert_eq!(cors.exposed_headers, ValidatedCorsList::Any);
    assert!(!cors.allow_credentials);
    assert_eq!(cors.max_age_seconds, None);
}

#[test]
fn validated_cors_rejects_invalid_inputs() {
    let mixed_origin = settings_with_cors(CorsSettings {
        allowed_origins: vec!["*".into(), "http://localhost:8082".into()],
        ..cors_settings()
    });
    let blank_header = settings_with_cors(CorsSettings {
        allowed_headers: vec!["Authorization".into(), "   ".into()],
        ..cors_settings()
    });
    let bad_method = settings_with_cors(CorsSettings {
        allowed_methods: vec!["NOT A METHOD".into()],
        ..cors_settings()
    });
    let bad_header = settings_with_cors(CorsSettings {
        exposed_headers: vec!["bad header".into()],
        ..cors_settings()
    });

    assert!(matches!(
        mixed_origin.validated_cors(),
        Err(SettingsError::MixedWildcardList("cors.allowed_origins"))
    ));
    assert!(matches!(
        blank_header.validated_cors(),
        Err(SettingsError::BlankListItem("cors.allowed_headers"))
    ));
    assert!(matches!(
        bad_method.validated_cors(),
        Err(SettingsError::InvalidHttpMethod {
            key: "cors.allowed_methods",
            ..
        })
    ));
    assert!(matches!(
        bad_header.validated_cors(),
        Err(SettingsError::InvalidHttpHeaderName {
            key: "cors.exposed_headers",
            ..
        })
    ));
}

#[test]
fn validated_cors_rejects_credentials_with_wildcard_allowlists() {
    let settings = settings_with_cors(CorsSettings {
        allowed_origins: vec!["*".into()],
        allow_credentials: true,
        ..cors_settings()
    });

    assert!(matches!(
        settings.validated_cors(),
        Err(SettingsError::WildcardCorsOrigin("cors.allowed_origins"))
    ));
}

#[test]
fn validated_cors_rejects_wildcard_and_non_http_origins() {
    let wildcard = settings_with_cors(CorsSettings {
        allowed_origins: vec!["*".into()],
        ..cors_settings()
    });
    let non_http = settings_with_cors(CorsSettings {
        allowed_origins: vec!["file:///tmp/admin.html".into()],
        ..cors_settings()
    });

    assert!(matches!(
        wildcard.validated_cors(),
        Err(SettingsError::WildcardCorsOrigin("cors.allowed_origins"))
    ));
    assert!(matches!(
        non_http.validated_cors(),
        Err(SettingsError::InvalidHttpOrigin {
            key: "cors.allowed_origins",
            ..
        })
    ));
}

#[test]
fn validated_cors_accepts_specific_methods_and_headers() {
    let settings = settings_with_cors(CorsSettings {
        allowed_origins: vec!["http://localhost:8082".into()],
        allowed_methods: vec!["get".into(), "post".into()],
        allowed_headers: vec!["authorization".into(), "content-type".into()],
        exposed_headers: vec!["x-request-id".into()],
        allow_credentials: false,
        max_age_seconds: Some(3600),
    });

    let cors = settings.validated_cors().unwrap();

    assert_eq!(cors.allowed_methods, ValidatedCorsList::Values(vec!["GET".into(), "POST".into()]));
    assert_eq!(
        cors.allowed_headers,
        ValidatedCorsList::Values(vec!["authorization".into(), "content-type".into()])
    );
    assert_eq!(cors.exposed_headers, ValidatedCorsList::Values(vec!["x-request-id".into()]));
    assert_eq!(cors.max_age_seconds, Some(3600));
}
