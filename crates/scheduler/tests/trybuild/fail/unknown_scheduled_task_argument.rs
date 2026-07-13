use scheduler_macros::scheduled_task;

#[scheduled_task(
    task_key = "task.one",
    name_key = "task.name",
    group = "SYSTEM",
    group_key = "group.system",
    description_key = "task.description",
    params = Params,
    unknown = "value",
)]
struct Task;

fn main() {}
