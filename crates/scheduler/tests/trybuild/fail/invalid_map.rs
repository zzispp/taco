use scheduler_macros::ScheduledTaskParams;
use std::collections::HashMap;

#[derive(ScheduledTaskParams)]
#[task_params(schema_version = 1)]
struct Params {
    values: HashMap<String, u32>,
}

fn main() {}
