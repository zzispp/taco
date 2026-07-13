use scheduler_macros::ScheduledTaskParams;

#[derive(ScheduledTaskParams)]
#[task_params(schema_version = 1)]
struct Params {
    #[param_field(widget = "text", widget = "text")]
    name: String,
}

fn main() {}
