use scheduler_macros::ScheduledTaskParams;

#[derive(ScheduledTaskParams)]
#[task_params(schema_version = 1)]
struct Params {
    enabled: bool,
    #[param_field(disabled_when_path = "enabled", disabled_when_values = [true || false])]
    name: String,
}

fn main() {}
