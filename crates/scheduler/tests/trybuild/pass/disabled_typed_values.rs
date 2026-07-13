use scheduler::application::task::TaskParams;
use scheduler_macros::ScheduledTaskParams;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, ScheduledTaskParams, Serialize)]
#[task_params(schema_version = 1)]
struct Params {
    enabled: bool,
    threshold: u32,
    #[param_field(disabled_when_path = "enabled", disabled_when_values = [false])]
    name: String,
    #[param_field(disabled_when_path = "threshold", disabled_when_values = [0, 3])]
    details: String,
}

fn main() {
    let _ = Params::form();
}
