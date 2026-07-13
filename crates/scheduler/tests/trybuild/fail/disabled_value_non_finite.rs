use scheduler_macros::ScheduledTaskParams;

#[derive(ScheduledTaskParams)]
#[task_params(schema_version = 1)]
struct Params {
    threshold: f64,
    #[param_field(disabled_when_path = "threshold", disabled_when_values = [1e999])]
    name: String,
}

fn main() {}
