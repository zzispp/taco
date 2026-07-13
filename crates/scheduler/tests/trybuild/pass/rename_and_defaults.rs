use scheduler::application::task::TaskParams;
use scheduler_macros::ScheduledTaskParams;
use serde::{Deserialize, Serialize};

fn default_retries() -> u16 {
    2
}

#[derive(Deserialize, ScheduledTaskParams, Serialize)]
#[task_params(schema_version = 1)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Params {
    #[serde(rename = "endpoint_url")]
    endpoint: String,
    #[serde(default = "default_retries")]
    retry_count: u16,
    #[serde(default)]
    note: Option<String>,
}

fn main() {
    let _ = Params::form();
    let _ = Params::default_params();
}
