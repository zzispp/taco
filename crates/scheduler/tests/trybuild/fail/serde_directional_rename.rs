use scheduler_macros::ScheduledTaskParams;

#[derive(ScheduledTaskParams)]
#[task_params(schema_version = 1)]
struct Params {
    #[serde(rename(serialize = "out", deserialize = "in"))]
    name: String,
}

fn main() {}
