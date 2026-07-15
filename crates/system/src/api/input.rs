use kernel::pagination::CursorPageRequest;
use serde::Deserialize;
use utoipa::IntoParams;

mod export_filters;
mod filters;
mod time_range;

pub(super) use export_filters::{config_export_filter, dict_data_export_filter, dict_type_export_filter, post_export_filter};
pub(super) use filters::{config_list_filter, dept_list_filter, dept_tree_filter, dict_data_list_filter, dict_type_list_filter, post_list_filter};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub(super) struct SystemListQuery {
    #[serde(default = "default_limit")]
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub limit: u64,
    #[serde(default)]
    pub cursor: Option<String>,
    pub dept_name: Option<String>,
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub post_code: Option<String>,
    pub post_name: Option<String>,
    pub remark: Option<String>,
    pub dict_name: Option<String>,
    pub dict_type: Option<String>,
    pub dict_label: Option<String>,
    pub config_name: Option<String>,
    pub config_key: Option<String>,
    pub config_type: Option<String>,
    pub public_read: Option<bool>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub(super) struct SystemExportQuery {
    pub post_code: Option<String>,
    pub post_name: Option<String>,
    pub remark: Option<String>,
    pub dict_name: Option<String>,
    pub dict_type: Option<String>,
    pub dict_label: Option<String>,
    pub config_name: Option<String>,
    pub config_key: Option<String>,
    pub config_type: Option<String>,
    pub public_read: Option<bool>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DeptTreeQuery {
    pub dept_name: Option<String>,
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

const fn default_limit() -> u64 {
    kernel::pagination::DEFAULT_CURSOR_LIMIT
}

pub(super) fn cursor_page(query: &SystemListQuery) -> CursorPageRequest {
    CursorPageRequest {
        limit: query.limit,
        cursor: query.cursor.clone(),
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;
    use types::http::RequestQuery;

    use super::SystemListQuery;

    #[tokio::test]
    async fn system_list_query_rejects_legacy_page_parameters_over_http() {
        async fn handler(RequestQuery(_query): RequestQuery<SystemListQuery>) {}

        for uri in ["/?page=1", "/?page_size=20", "/?page=1&page_size=20"] {
            let response = Router::new()
                .route("/", get(handler))
                .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST, "uri={uri}");
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(body["code"], "invalid_input", "uri={uri}");
        }
    }
}
