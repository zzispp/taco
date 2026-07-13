use quote::quote;

use crate::{
    serde_attr::SerdeDefault,
    task_params_attr::{label_key, required_schema_version},
    task_params_model::{FieldModel, ParamsModel},
    task_params_type::SchemaType,
};

pub fn implementation(model: &ParamsModel) -> syn::Result<proc_macro2::TokenStream> {
    let schema_version = required_schema_version(&model.attrs)?;
    let form = form_tokens(model);
    let defaults = default_tokens(model);
    let validate = validate_tokens(model);
    let render = render_tokens(model);
    Ok(quote! {
        const SCHEMA_VERSION: i16 = #schema_version;

        fn form() -> scheduler::domain::TaskParamFormSpec {
            #form
        }

        fn default_params() -> serde_json::Value {
            #defaults
        }

        fn validate(value: &serde_json::Value) -> scheduler::application::SchedulerResult<()> {
            #validate
        }

        fn render_invoke_target(
            task_key: &str,
            value: &serde_json::Value,
        ) -> scheduler::application::SchedulerResult<String> {
            #render
        }
    })
}

fn form_tokens(model: &ParamsModel) -> proc_macro2::TokenStream {
    let property_inserts = model.fields.iter().map(property_insert);
    let required = model
        .fields
        .iter()
        .filter(|field| field.required(model.serde.default.as_ref()))
        .map(|field| field.wire_name.as_str());
    let ui_fields = model.fields.iter().map(ui_field);
    quote! {
        let mut properties = std::collections::BTreeMap::new();
        #(#property_inserts)*
        scheduler::domain::TaskParamFormSpec {
            schema_version: Self::SCHEMA_VERSION,
            schema: scheduler::domain::ParamSchema::Object(scheduler::domain::ObjectParamSchema {
                properties,
                required: vec![#(#required.into()),*],
                additional_properties: false,
            }),
            ui: scheduler::domain::ParamUiSpec { fields: vec![#(#ui_fields),*] },
        }
    }
}

fn property_insert(field: &FieldModel) -> proc_macro2::TokenStream {
    let name = &field.wire_name;
    let options = field.attrs.options.as_deref().unwrap_or_default();
    let schema = field.field_type.schema.schema_tokens(field.attrs.pattern.as_ref(), options);
    quote! { properties.insert(#name.into(), #schema); }
}

fn ui_field(field: &FieldModel) -> proc_macro2::TokenStream {
    let path = &field.wire_name;
    let label = label_key(path, &field.attrs);
    let widget = field.widget.tokens();
    let placeholder = optional_key(field.attrs.placeholder_key.as_ref());
    let help = optional_key(field.attrs.help_key.as_ref());
    let options = field.attrs.options.as_deref().unwrap_or_default();
    let disabled = disabled_when(field);
    quote! {
        scheduler::domain::ParamFieldSpec {
            path: #path.into(),
            label_key: #label.into(),
            widget: #widget,
            placeholder_key: #placeholder,
            help_key: #help,
            options: vec![#(#options.into()),*],
            disabled_when: #disabled,
        }
    }
}

fn optional_key(value: Option<&syn::LitStr>) -> proc_macro2::TokenStream {
    match value {
        Some(value) => quote! { Some(#value.into()) },
        None => quote! { None },
    }
}

fn disabled_when(field: &FieldModel) -> proc_macro2::TokenStream {
    let Some(path) = field.disabled_wire_path.as_ref() else {
        return quote! { None };
    };
    let values = field
        .attrs
        .disabled_when_values
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(crate::json_scalar::JsonScalar::json_tokens);
    quote! {
        Some(scheduler::domain::ParamCondition {
            path: #path.into(),
            values: vec![#(#values),*],
        })
    }
}

fn default_tokens(model: &ParamsModel) -> proc_macro2::TokenStream {
    let needs_container = model.serde.default.is_some()
        && model
            .fields
            .iter()
            .any(|field| !field.field_type.optional && field.attrs.default.is_none() && field.serde.default.is_none());
    let container = needs_container.then(|| container_default(model.serde.default.as_ref().expect("presence checked")));
    let entries = model.fields.iter().filter_map(|field| default_entry(field, needs_container));
    quote! {
        #container
        let mut params = serde_json::Map::new();
        #(#entries)*
        serde_json::Value::Object(params)
    }
}

fn container_default(default: &SerdeDefault) -> proc_macro2::TokenStream {
    let value = match default {
        SerdeDefault::DefaultTrait => quote! { <Self as Default>::default() },
        SerdeDefault::Function(path) => quote! { #path() },
    };
    quote! { let __scheduler_container_default: Self = #value; }
}

fn default_entry(field: &FieldModel, has_container: bool) -> Option<proc_macro2::TokenStream> {
    if field.field_type.optional && field.attrs.default.is_none() {
        return None;
    }
    let name = &field.wire_name;
    let value = default_value(field, has_container);
    Some(quote! { params.insert(#name.into(), serde_json::json!(#value)); })
}

fn default_value(field: &FieldModel, has_container: bool) -> proc_macro2::TokenStream {
    if let Some(value) = field.attrs.default.as_ref() {
        return quote! { #value };
    }
    match field.serde.default.as_ref() {
        Some(SerdeDefault::DefaultTrait) => {
            let ty = &field.ty;
            quote! { <#ty as Default>::default() }
        }
        Some(SerdeDefault::Function(path)) => quote! { #path() },
        None if has_container => {
            let ident = &field.ident;
            quote! { &__scheduler_container_default.#ident }
        }
        None => {
            let ty = &field.ty;
            quote! { <#ty as Default>::default() }
        }
    }
}

fn validate_tokens(model: &ParamsModel) -> proc_macro2::TokenStream {
    let names = model.fields.iter().map(|field| field.wire_name.as_str());
    let required = model
        .fields
        .iter()
        .filter(|field| field.required(model.serde.default.as_ref()))
        .map(|field| field.wire_name.as_str());
    let enums = model
        .fields
        .iter()
        .filter(|field| matches!(field.field_type.schema, SchemaType::String) && field.attrs.options.as_ref().is_some_and(|v| !v.is_empty()))
        .map(enum_check);
    let patterns = model
        .fields
        .iter()
        .filter(|field| matches!(field.field_type.schema, SchemaType::String) && field.attrs.pattern.is_some())
        .map(pattern_check);
    let custom = model.attrs.validate_with.as_ref().map(|path| quote! { #path(&params)?; });
    quote! {
        scheduler::application::task::validate_param_object_keys(value, &[#(#names),*])?;
        scheduler::application::task::validate_required_param_fields(value, &[#(#required),*])?;
        let params: Self = serde_json::from_value(value.clone())
            .map_err(|_| scheduler::application::task::invalid_task_params())?;
        #(#enums)*
        #(#patterns)*
        #custom
        Ok(())
    }
}

fn enum_check(field: &FieldModel) -> proc_macro2::TokenStream {
    let ident = &field.ident;
    let options = field.attrs.options.as_deref().unwrap_or_default();
    if field.field_type.optional {
        return quote! {
            if let Some(value) = &params.#ident {
                scheduler::application::task::validate_param_enum(value, &[#(#options),*])?;
            }
        };
    }
    quote! { scheduler::application::task::validate_param_enum(&params.#ident, &[#(#options),*])?; }
}

fn pattern_check(field: &FieldModel) -> proc_macro2::TokenStream {
    let ident = &field.ident;
    let pattern = field.attrs.pattern.as_ref().expect("caller filters patterns");
    if field.field_type.optional {
        return quote! {
            if let Some(value) = &params.#ident {
                scheduler::application::task::validate_param_pattern(value, #pattern)?;
            }
        };
    }
    quote! { scheduler::application::task::validate_param_pattern(&params.#ident, #pattern)?; }
}

fn render_tokens(model: &ParamsModel) -> proc_macro2::TokenStream {
    if let Some(path) = model.attrs.render_with.as_ref() {
        return quote! {
            Self::validate(value)?;
            let params: Self = serde_json::from_value(value.clone())
                .map_err(|_| scheduler::application::task::invalid_task_params())?;
            #path(task_key, &params)
        };
    }
    quote! {
        Self::validate(value)?;
        let payload = serde_json::to_string(value)
            .map_err(|_| scheduler::application::task::invalid_task_params())?;
        Ok(format!("{task_key}({payload})"))
    }
}
