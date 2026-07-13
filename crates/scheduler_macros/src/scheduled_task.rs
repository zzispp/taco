use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ItemStruct, Token, Type, parse::Parse, parse::ParseStream, parse_macro_input};

pub fn expand(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as ScheduledTaskArgs);
    let item = parse_macro_input!(input as ItemStruct);
    expand_item(args, item).into()
}

fn expand_item(args: ScheduledTaskArgs, item: ItemStruct) -> proc_macro2::TokenStream {
    let ident = &item.ident;
    let task_key = args.task_key;
    let name_key = args.name_key;
    let group = args.group;
    let group_key = args.group_key;
    let description_key = args.description_key;
    let repeatable = args.repeatable.unwrap_or_else(|| syn::parse_quote!(false));
    let params = args.params;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    quote! {
        #item

        impl #impl_generics scheduler::application::task::ScheduledTaskMetadata for #ident #ty_generics #where_clause {
            fn descriptor() -> scheduler::application::task::ScheduledTaskDefinition {
                scheduler::application::task::ScheduledTaskDefinition {
                    task_key: #task_key,
                    name_key: #name_key,
                    group: #group,
                    group_key: #group_key,
                    description_key: #description_key,
                    repeatable: #repeatable,
                    params: scheduler::application::task::ParamDefinition {
                        schema_version: <#params as scheduler::application::task::TaskParams>::SCHEMA_VERSION,
                        form: <#params as scheduler::application::task::TaskParams>::form,
                        default_params: <#params as scheduler::application::task::TaskParams>::default_params,
                        validate: <#params as scheduler::application::task::TaskParams>::validate,
                        render_invoke_target: <#params as scheduler::application::task::TaskParams>::render_invoke_target,
                    },
                    factory: || std::sync::Arc::new(<#ident #ty_generics as Default>::default()),
                }
            }
        }
    }
}

struct ScheduledTaskArgs {
    task_key: Expr,
    name_key: Expr,
    group: Expr,
    group_key: Expr,
    description_key: Expr,
    repeatable: Option<Expr>,
    params: Type,
}

impl Parse for ScheduledTaskArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = PartialScheduledTaskArgs::default();
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "task_key" => set_once(&mut args.task_key, input.parse()?, &key)?,
                "name_key" => set_once(&mut args.name_key, input.parse()?, &key)?,
                "group" => set_once(&mut args.group, input.parse()?, &key)?,
                "group_key" => set_once(&mut args.group_key, input.parse()?, &key)?,
                "description_key" => set_once(&mut args.description_key, input.parse()?, &key)?,
                "repeatable" => set_once(&mut args.repeatable, input.parse()?, &key)?,
                "params" => set_once(&mut args.params, input.parse()?, &key)?,
                other => return Err(syn::Error::new(key.span(), format!("unsupported scheduled_task argument: {other}"))),
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }
        args.finish(input)
    }
}

#[derive(Default)]
struct PartialScheduledTaskArgs {
    task_key: Option<Expr>,
    name_key: Option<Expr>,
    group: Option<Expr>,
    group_key: Option<Expr>,
    description_key: Option<Expr>,
    repeatable: Option<Expr>,
    params: Option<Type>,
}

impl PartialScheduledTaskArgs {
    fn finish(self, input: ParseStream<'_>) -> syn::Result<ScheduledTaskArgs> {
        Ok(ScheduledTaskArgs {
            task_key: required(input, self.task_key, "task_key")?,
            name_key: required(input, self.name_key, "name_key")?,
            group: required(input, self.group, "group")?,
            group_key: required(input, self.group_key, "group_key")?,
            description_key: required(input, self.description_key, "description_key")?,
            repeatable: self.repeatable,
            params: required(input, self.params, "params")?,
        })
    }
}

fn set_once<T>(slot: &mut Option<T>, value: T, key: &syn::Ident) -> syn::Result<()> {
    if slot.is_some() {
        return Err(syn::Error::new(key.span(), format!("duplicate scheduled_task argument: {key}")));
    }
    *slot = Some(value);
    Ok(())
}

fn required<T>(input: ParseStream<'_>, value: Option<T>, key: &'static str) -> syn::Result<T> {
    value.ok_or_else(|| input.error(format!("{key} is required")))
}

#[cfg(test)]
mod tests {
    use super::{ScheduledTaskArgs, expand_item};

    const REQUIRED: &str = r#"
        task_key = "task.key",
        name_key = "task.name",
        group = "SYSTEM",
        group_key = "group.system",
        description_key = "task.description",
        params = Params,
    "#;

    #[test]
    fn generates_explicit_metadata_without_inventory() {
        let args = syn::parse_str::<ScheduledTaskArgs>(REQUIRED).expect("arguments should parse");
        let item = syn::parse_quote!(
            #[derive(Default)]
            struct Task;
        );
        let generated = expand_item(args, item).to_string();
        assert!(generated.contains("ScheduledTaskMetadata"));
        assert!(generated.contains("fn descriptor"));
        assert!(!generated.contains("inventory"));
    }

    #[test]
    fn rejects_missing_comma_duplicate_and_unknown_arguments() {
        assert!(syn::parse_str::<ScheduledTaskArgs>(&REQUIRED.replace("task.name\",", "task.name\" ")).is_err());
        assert!(syn::parse_str::<ScheduledTaskArgs>(&format!("{REQUIRED} task_key = \"duplicate\", ")).is_err());
        assert!(syn::parse_str::<ScheduledTaskArgs>(&format!("{REQUIRED} unknown = \"value\", ")).is_err());
    }
}
