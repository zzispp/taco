use scheduler::application::task::TaskParams;
use scheduler_macros::ScheduledTaskParams;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, ScheduledTaskParams, Serialize)]
#[task_params(schema_version = 1)]
struct Params {
    numbers: Vec<u32>,
    flags: Vec<bool>,
}

fn main() {
    let _ = Params::form();
}
