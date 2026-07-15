use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    application::{SchedulerError, SchedulerResult, task::TaskParams},
    domain::{ObjectParamSchema, ParamSchema, ParamUiSpec, TaskParamFormSpec},
};
use scheduler_macros::ScheduledTaskParams;

use super::http_sanitization::{sanitize_http_method, sanitize_http_url};

const HTTP_PARAMS_SCHEMA_VERSION: i16 = 1;
const NO_PARAMS_SCHEMA_VERSION: i16 = 1;

#[derive(Clone, Debug, Default)]
pub struct NoTaskParams;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, ScheduledTaskParams, Serialize)]
#[task_params(schema_version = HTTP_PARAMS_SCHEMA_VERSION, render_with = Self::render_http)]
pub struct HttpRequestParams {
    #[param_field(
        required,
        widget = "select",
        label_key = "scheduler.param_fields.http.method",
        options = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"],
        default = "GET"
    )]
    pub method: String,
    #[param_field(
        required,
        widget = "text",
        label_key = "scheduler.param_fields.http.url",
        pattern = r"^https?://.+",
        default = ""
    )]
    pub url: String,
    #[serde(default)]
    #[param_field(widget = "key_value", label_key = "scheduler.param_fields.http.headers")]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    #[param_field(
        widget = "textarea",
        label_key = "scheduler.param_fields.http.body",
        default = "",
        disabled_when_path = "method",
        disabled_when_values = ["GET", "HEAD"]
    )]
    pub body: Option<String>,
}

impl TaskParams for NoTaskParams {
    const SCHEMA_VERSION: i16 = NO_PARAMS_SCHEMA_VERSION;

    fn form() -> TaskParamFormSpec {
        TaskParamFormSpec {
            schema_version: Self::SCHEMA_VERSION,
            schema: ParamSchema::Object(ObjectParamSchema {
                properties: Default::default(),
                required: Vec::new(),
                additional_properties: false,
            }),
            ui: ParamUiSpec::default(),
        }
    }

    fn default_params() -> Value {
        json!({})
    }

    fn validate(value: &Value) -> SchedulerResult<()> {
        match value.as_object() {
            Some(object) if object.is_empty() => Ok(()),
            _ => Err(invalid_params()),
        }
    }

    fn render_invoke_target(task_key: &str, value: &Value) -> SchedulerResult<String> {
        Self::validate(value)?;
        Ok(format!("{task_key}()"))
    }
}

pub fn is_body_method(method: &str) -> bool {
    !method.eq_ignore_ascii_case("GET") && !method.eq_ignore_ascii_case("HEAD")
}

fn invalid_params() -> SchedulerError {
    SchedulerError::InvalidInput(kernel::error::LocalizedError::new("errors.scheduler.invalid_params"))
}

impl HttpRequestParams {
    fn render_http(task_key: &str, params: &Self) -> SchedulerResult<String> {
        Ok(format!(
            "{task_key}({}, {})",
            sanitize_http_method(&params.method),
            sanitize_http_url(&params.url)
        ))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::application::task::TaskParams;

    use super::HttpRequestParams;

    #[test]
    fn http_invoke_target_drops_url_userinfo_and_query_parameters() {
        let target = <HttpRequestParams as TaskParams>::render_invoke_target(
            "httpClient.request",
            &json!({
                "method": "POST",
                "url": "https://url-user:url-password@example.test/run?token=query-token#fragment",
                "headers": {},
                "body": null,
            }),
        )
        .unwrap();

        assert_eq!(target, "httpClient.request(POST, https://example.test/run)");
        for marker in ["url-user", "url-password", "query-token", "fragment"] {
            assert!(!target.contains(marker), "invoke target leaked {marker}");
        }
    }
}
