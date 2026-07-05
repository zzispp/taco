use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, LitStr, Token, parse::Parse, parse::ParseStream, parse_macro_input};

#[proc_macro_attribute]
pub fn require_perms(args: TokenStream, input: TokenStream) -> TokenStream {
    let permission = parse_macro_input!(args as LitStr);
    let function = parse_macro_input!(input as ItemFn);
    let name = function.sig.ident.to_string();
    let expanded = quote! {
        #function

        inventory::submit! {
            types::rbac::ProtectedHandler {
                function: #name,
                permission: #permission,
            }
        }
    };
    expanded.into()
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

        inventory::submit! {
            types::rbac::DataScopeHandler {
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
        let mut dept_alias: Option<LitStr> = None;
        let mut user_alias: Option<LitStr> = None;
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;
            match key.to_string().as_str() {
                "dept_alias" => dept_alias = Some(value),
                "user_alias" => user_alias = Some(value),
                other => return Err(syn::Error::new(key.span(), format!("unsupported data_scope argument: {other}"))),
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self {
            dept_alias: dept_alias.ok_or_else(|| input.error("dept_alias is required"))?,
            user_alias: user_alias.ok_or_else(|| input.error("user_alias is required"))?,
        })
    }
}
