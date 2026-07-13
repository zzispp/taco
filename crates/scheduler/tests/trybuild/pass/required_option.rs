use scheduler::application::task::TaskParams;
use scheduler_macros::ScheduledTaskParams;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, ScheduledTaskParams, Serialize)]
#[task_params(schema_version = 1)]
struct Params {
    #[param_field(required)]
    note: Option<String>,
}

fn main() {
    let _ = Params::validate(&serde_json::json!({"note": "present"}));
}
