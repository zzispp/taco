use scheduler_macros::ScheduledTaskParams;

#[derive(ScheduledTaskParams)]
#[task_params(schema_version = 1)]
#[serde(bound = "")]
struct Params {
    name: String,
}

fn main() {}
