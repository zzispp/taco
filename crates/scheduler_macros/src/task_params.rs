use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

use crate::{task_params_codegen, task_params_model::ParamsModel};

pub fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_input(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

fn expand_input(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let fields = named_fields(&input)?;
    let model = ParamsModel::parse(&input, fields)?;
    let implementation = task_params_codegen::implementation(&model)?;
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics scheduler::application::task::TaskParams for #ident #ty_generics #where_clause {
            #implementation
        }
    })
}

fn named_fields(input: &DeriveInput) -> syn::Result<Vec<syn::Field>> {
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(input, "ScheduledTaskParams supports structs only"));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(input, "ScheduledTaskParams requires named fields"));
    };
    Ok(fields.named.iter().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::expand_input;

    #[test]
    fn generated_contract_uses_application_task_and_wire_keys() {
        let input = syn::parse_quote! {
            #[task_params(schema_version = VERSION)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[param_field(required)] optional_value: Option<u32>,
                values: Vec<bool>,
            }
        };
        let generated = expand_input(input).expect("derive should expand").to_string();
        assert!(generated.contains("scheduler :: application :: task :: TaskParams"));
        assert!(generated.contains("const SCHEMA_VERSION : i16 = VERSION"));
        assert!(generated.contains("optionalValue"));
        assert!(generated.contains("ParamWidget :: Number"));
        assert!(generated.contains("ParamSchema :: Boolean"));
    }

    #[test]
    fn optional_without_explicit_default_is_omitted() {
        let input = syn::parse_quote! {
            #[task_params(schema_version = 1)]
            struct Params { #[serde(default)] note: Option<String> }
        };
        let generated = expand_input(input).expect("derive should expand").to_string();
        assert!(!generated.contains("params . insert (\"note\""));
    }

    #[test]
    fn required_option_checks_raw_json() {
        let input = syn::parse_quote! {
            #[task_params(schema_version = 1)]
            struct Params { #[param_field(required)] note: Option<String> }
        };
        let generated = expand_input(input).expect("derive should expand").to_string();
        assert!(generated.contains("validate_required_param_fields"));
        assert!(generated.contains("[\"note\"]"));
    }
}
