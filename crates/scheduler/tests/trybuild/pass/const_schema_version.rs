use scheduler::application::task::TaskParams;
use scheduler_macros::ScheduledTaskParams;
use serde::{Deserialize, Serialize};

const BASE_VERSION: i16 = 4;
const SCHEMA_VERSION: i16 = BASE_VERSION + 1;

#[derive(Deserialize, ScheduledTaskParams, Serialize)]
#[task_params(schema_version = SCHEMA_VERSION)]
struct Params {
    name: String,
}

fn main() {
    assert_eq!(Params::SCHEMA_VERSION, SCHEMA_VERSION);
}
