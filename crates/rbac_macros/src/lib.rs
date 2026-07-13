use proc_macro::TokenStream;
use std::collections::HashSet;

use quote::quote;
use syn::{ItemFn, LitStr, Token, parse::Parse, parse::ParseStream, parse_macro_input, punctuated::Punctuated};

#[proc_macro_attribute]
pub fn require_perms(args: TokenStream, input: TokenStream) -> TokenStream {
    protected_handler(args, input, RequirementKind::AllOf)
}

#[proc_macro_attribute]
pub fn require_any_perms(args: TokenStream, input: TokenStream) -> TokenStream {
    protected_handler(args, input, RequirementKind::AnyOf)
}

#[proc_macro_attribute]
pub fn data_scope(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as DataScopeArgs);
    let function = parse_macro_input!(input as ItemFn);
    let name = function.sig.ident.to_string();
    let dept_alias = args.dept_alias;
    let user_alias = args.user_alias;
    let expanded = quote! {
        #function

        ::inventory::submit! {
            ::types::rbac::DataScopeHandler {
                function: #name,
                dept_alias: #dept_alias,
                user_alias: #user_alias,
            }
        }
    };
    expanded.into()
}

struct DataScopeArgs {
    dept_alias: LitStr,
    user_alias: LitStr,
}

impl Parse for DataScopeArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let entries = Punctuated::<NamedString, Token![,]>::parse_terminated(input)?;
        let mut dept_alias: Option<LitStr> = None;
        let mut user_alias: Option<LitStr> = None;
        for entry in entries {
            assign_data_scope_entry(entry, &mut dept_alias, &mut user_alias)?;
        }
        Ok(Self {
            dept_alias: dept_alias.ok_or_else(|| input.error("dept_alias is required"))?,
            user_alias: user_alias.ok_or_else(|| input.error("user_alias is required"))?,
        })
    }
}

#[derive(Clone, Copy)]
enum RequirementKind {
    AllOf,
    AnyOf,
}

struct PermissionArgs {
    values: Punctuated<LitStr, Token![,]>,
}

impl Parse for PermissionArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let values = Punctuated::<LitStr, Token![,]>::parse_terminated(input)?;
        if values.is_empty() {
            return Err(input.error("at least one permission is required"));
        }
        let mut unique = HashSet::new();
        for value in &values {
            if value.value().trim().is_empty() {
                return Err(syn::Error::new(value.span(), "permission cannot be blank"));
            }
            if !unique.insert(value.value()) {
                return Err(syn::Error::new(value.span(), "duplicate permission"));
            }
        }
        Ok(Self { values })
    }
}

struct NamedString {
    key: syn::Ident,
    value: LitStr,
}

impl Parse for NamedString {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            key: input.parse()?,
            value: {
                input.parse::<Token![=]>()?;
                input.parse()?
            },
        })
    }
}

fn protected_handler(args: TokenStream, input: TokenStream, kind: RequirementKind) -> TokenStream {
    let args = parse_macro_input!(args as PermissionArgs);
    let function = parse_macro_input!(input as ItemFn);
    let name = function.sig.ident.to_string();
    let permissions = args.values;
    let requirement = match kind {
        RequirementKind::AllOf => quote!(rbac::application::PermissionRequirement::AllOf(&[#permissions])),
        RequirementKind::AnyOf => quote!(rbac::application::PermissionRequirement::AnyOf(&[#permissions])),
    };
    quote! {
        #function

        ::rbac::inventory::submit! {
            ::rbac::application::ProtectedHandler {
                function: #name,
                requirement: #requirement,
            }
        }
    }
    .into()
}

fn assign_data_scope_entry(entry: NamedString, dept_alias: &mut Option<LitStr>, user_alias: &mut Option<LitStr>) -> syn::Result<()> {
    let target = match entry.key.to_string().as_str() {
        "dept_alias" => dept_alias,
        "user_alias" => user_alias,
        other => return Err(syn::Error::new(entry.key.span(), format!("unsupported data_scope argument: {other}"))),
    };
    if target.is_some() {
        return Err(syn::Error::new(entry.key.span(), "duplicate data_scope argument"));
    }
    *target = Some(entry.value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DataScopeArgs, PermissionArgs};

    #[test]
    fn permission_args_require_non_empty_unique_comma_separated_values() {
        assert!(syn::parse_str::<PermissionArgs>(r#""job:import", "job:edit""#).is_ok());
        assert!(syn::parse_str::<PermissionArgs>("").is_err());
        assert!(syn::parse_str::<PermissionArgs>(r#""job:import" "job:edit""#).is_err());
        assert!(syn::parse_str::<PermissionArgs>(r#""job:import", "job:import""#).is_err());
        assert!(syn::parse_str::<PermissionArgs>(r#""   ""#).is_err());
    }

    #[test]
    fn data_scope_args_reject_duplicates_and_missing_commas() {
        assert!(syn::parse_str::<DataScopeArgs>(r#"dept_alias = "d", user_alias = "u""#).is_ok());
        assert!(syn::parse_str::<DataScopeArgs>(r#"dept_alias = "d" user_alias = "u""#).is_err());
        assert!(syn::parse_str::<DataScopeArgs>(r#"dept_alias = "d", dept_alias = "x", user_alias = "u""#).is_err());
    }
}
