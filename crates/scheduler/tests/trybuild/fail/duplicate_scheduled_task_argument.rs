use scheduler_macros::scheduled_task;

#[scheduled_task(
    task_key = "task.one",
    task_key = "task.two",
    name_key = "task.name",
    group = "SYSTEM",
    group_key = "group.system",
    description_key = "task.description",
    params = Params,
)]
struct Task;

fn main() {}
