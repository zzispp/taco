use scheduler_macros::ScheduledTaskParams;

struct CustomValue;

#[derive(ScheduledTaskParams)]
#[task_params(schema_version = 1)]
struct Params {
    value: CustomValue,
}

fn main() {}
