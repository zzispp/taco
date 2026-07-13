use proc_macro::TokenStream;

mod json_scalar;
mod scheduled_task;
mod serde_attr;
mod task_params;
mod task_params_attr;
mod task_params_codegen;
mod task_params_model;
mod task_params_type;

#[proc_macro_attribute]
pub fn scheduled_task(args: TokenStream, input: TokenStream) -> TokenStream {
    scheduled_task::expand(args, input)
}

#[proc_macro_derive(ScheduledTaskParams, attributes(task_params, param_field, serde))]
pub fn scheduled_task_params(input: TokenStream) -> TokenStream {
    task_params::expand(input)
}
