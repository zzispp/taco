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

#[cfg(test)]
mod tests {
    use super::PermissionArgs;

    #[test]
    fn permission_args_require_non_empty_unique_comma_separated_values() {
        assert!(syn::parse_str::<PermissionArgs>(r#""job:import", "job:edit""#).is_ok());
        assert!(syn::parse_str::<PermissionArgs>("").is_err());
        assert!(syn::parse_str::<PermissionArgs>(r#""job:import" "job:edit""#).is_err());
        assert!(syn::parse_str::<PermissionArgs>(r#""job:import", "job:import""#).is_err());
        assert!(syn::parse_str::<PermissionArgs>(r#""   ""#).is_err());
    }
}
