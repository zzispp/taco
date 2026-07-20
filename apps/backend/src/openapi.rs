use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::system::health,
        crate::system::ready
    ),
    nest(
        (path = "/api", api = audit::api::AuditApiDoc),
        (path = "/api", api = observability::api::SystemLogApiDoc)
    ),
    tags(
        (name = "system", description = "System endpoints")
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::{Value, json};
    use utoipa::OpenApi;

    use super::ApiDoc;

    const AUDIT_PATHS: [&str; 11] = [
        "/api/system/login-logs",
        "/api/system/login-logs/batch",
        "/api/system/login-logs/clean",
        "/api/system/login-logs/export",
        "/api/system/login-logs/{id}",
        "/api/system/login-logs/{username}/unlock",
        "/api/system/operation-logs",
        "/api/system/operation-logs/batch",
        "/api/system/operation-logs/clean",
        "/api/system/operation-logs/export",
        "/api/system/operation-logs/{id}",
    ];
    const SYSTEM_LOG_PATHS: [&str; 7] = [
        "/api/system/system-logs",
        "/api/system/system-logs/batch",
        "/api/system/system-logs/clean",
        "/api/system/system-logs/clean/count",
        "/api/system/system-logs/clean/executions/{execution_id}",
        "/api/system/system-logs/export",
        "/api/system/system-logs/{id}",
    ];
    const HTTP_METHODS: [&str; 5] = ["delete", "get", "patch", "post", "put"];
    const XLSX_CONTENT_TYPE: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

    #[test]
    fn audit_contract_has_every_path_method_and_bearer_requirement() {
        let document = document();
        let audit_paths = audit_paths(&document);

        assert_eq!(audit_paths.keys().copied().collect::<BTreeSet<_>>(), BTreeSet::from(AUDIT_PATHS));
        assert_eq!(operation_count(&audit_paths), 12);
        assert_bearer_security(&document, &audit_paths);
        assert!(document.get("security").is_none());
        assert!(document["paths"]["/health"]["get"].get("security").is_none());
    }

    #[test]
    fn audit_contract_reuses_error_schema_and_publishes_xlsx_media_type() {
        let document = document();
        let audit_paths = audit_paths(&document);

        for schema in [
            "ApiErrorResponse",
            "BatchIdsRequest",
            "LoginLogResponse",
            "OperationLogDetailResponse",
            "OperationLogSummaryResponse",
        ] {
            assert!(document.pointer(&format!("/components/schemas/{schema}")).is_some(), "missing schema {schema}");
        }
        for path in ["/api/system/operation-logs/export", "/api/system/login-logs/export"] {
            assert!(audit_paths[path]["post"]["responses"]["200"]["content"].get(XLSX_CONTENT_TYPE).is_some());
        }
        assert_eq!(
            audit_paths["/api/system/login-logs/{username}/unlock"]["put"]["responses"]["404"]["content"]["application/json"]["schema"]["$ref"],
            json!("#/components/schemas/ApiErrorResponse")
        );
    }

    #[test]
    fn audit_contract_distinguishes_list_and_export_query_parameters() {
        let document = document();
        let audit_paths = audit_paths(&document);

        assert_query_parameters(
            audit_paths["/api/system/operation-logs"],
            "get",
            &[
                "begin_time",
                "business_type",
                "cursor",
                "end_time",
                "limit",
                "oper_ip",
                "oper_name",
                "sort_by",
                "sort_order",
                "status",
                "title",
            ],
        );
        assert_query_parameters(
            audit_paths["/api/system/login-logs"],
            "get",
            &[
                "begin_time",
                "cursor",
                "end_time",
                "event_type",
                "ipaddr",
                "limit",
                "sort_by",
                "sort_order",
                "status",
                "user_name",
            ],
        );
        for path in ["/api/system/operation-logs/export", "/api/system/login-logs/export"] {
            let names = parameter_names(&audit_paths[path]["post"]);
            assert!(!names.contains("page"));
            assert!(!names.contains("page_size"));
        }
    }

    #[test]
    fn system_log_contract_has_every_management_route_and_xlsx_export() {
        let document = document();
        let paths = system_log_paths(&document);

        assert_eq!(paths.keys().copied().collect::<BTreeSet<_>>(), BTreeSet::from(SYSTEM_LOG_PATHS));
        assert_eq!(operation_count(&paths), 8);
        assert_bearer_security(&document, &paths);
        assert!(
            paths["/api/system/system-logs/export"]["post"]["responses"]["200"]["content"]
                .get(XLSX_CONTENT_TYPE)
                .is_some()
        );
        for schema in [
            "SystemLogSummaryResponse",
            "SystemLogDetailResponse",
            "SystemLogCleanupCountResponse",
            "SystemLogCleanupAcceptedResponse",
            "SystemLogCleanupExecutionResponse",
        ] {
            assert!(document.pointer(&format!("/components/schemas/{schema}")).is_some(), "missing schema {schema}");
        }
    }

    fn document() -> Value {
        serde_json::to_value(ApiDoc::openapi()).unwrap()
    }

    fn audit_paths(document: &Value) -> std::collections::BTreeMap<&str, &Value> {
        document["paths"]
            .as_object()
            .unwrap()
            .iter()
            .filter_map(|(path, item)| {
                (path.starts_with("/api/system/operation-logs") || path.starts_with("/api/system/login-logs")).then_some((path.as_str(), item))
            })
            .collect()
    }

    fn system_log_paths(document: &Value) -> std::collections::BTreeMap<&str, &Value> {
        document["paths"]
            .as_object()
            .unwrap()
            .iter()
            .filter_map(|(path, item)| path.starts_with("/api/system/system-logs").then_some((path.as_str(), item)))
            .collect()
    }

    fn operation_count(paths: &std::collections::BTreeMap<&str, &Value>) -> usize {
        paths
            .values()
            .map(|path| HTTP_METHODS.iter().filter(|method| path.get(**method).is_some()).count())
            .sum()
    }

    fn assert_query_parameters(path: &Value, method: &str, expected: &[&str]) {
        let operation = &path[method];
        assert_eq!(parameter_names(operation), expected.iter().copied().collect());
        let names = parameter_names(operation);
        assert!(!names.contains("page"));
        assert!(!names.contains("page_size"));
        for name in ["cursor", "limit"] {
            let parameter = operation["parameters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|parameter| parameter["name"] == name)
                .unwrap();
            assert_eq!(parameter["required"], false);
        }
    }

    fn parameter_names(operation: &Value) -> BTreeSet<&str> {
        operation["parameters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|parameter| parameter["name"].as_str().unwrap())
            .collect()
    }

    fn assert_bearer_security(document: &Value, paths: &std::collections::BTreeMap<&str, &Value>) {
        assert_eq!(document.pointer("/components/securitySchemes/bearerAuth/type"), Some(&json!("http")));
        assert_eq!(document.pointer("/components/securitySchemes/bearerAuth/scheme"), Some(&json!("bearer")));
        for path in paths.values() {
            for method in HTTP_METHODS.iter().filter_map(|method| path.get(*method)) {
                assert_eq!(method["security"], json!([{ "bearerAuth": [] }]));
                for status in ["401", "403", "503"] {
                    assert_eq!(
                        method["responses"][status]["content"]["application/json"]["schema"]["$ref"],
                        json!("#/components/schemas/ApiErrorResponse")
                    );
                }
            }
        }
    }
}
