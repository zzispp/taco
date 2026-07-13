use scheduler_macros::ScheduledTaskParams;

#[derive(ScheduledTaskParams)]
#[task_params(schema_version = 1)]
#[serde(tag = "kind")]
struct Params {
    name: String,
}

fn main() {}
