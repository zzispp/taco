use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Expr, ExprArray, ExprLit, ExprPath, Lit, LitStr, meta::ParseNestedMeta};

use crate::json_scalar::JsonScalar;

#[derive(Default)]
pub struct ContainerAttrs {
    pub schema_version: Option<Expr>,
    pub validate_with: Option<ExprPath>,
    pub render_with: Option<ExprPath>,
}

#[derive(Default)]
pub struct FieldAttrs {
    pub required: bool,
    pub widget: Option<Widget>,
    pub label_key: Option<LitStr>,
    pub placeholder_key: Option<LitStr>,
    pub help_key: Option<LitStr>,
    pub pattern: Option<LitStr>,
    pub options: Option<Vec<LitStr>>,
    pub default: Option<Expr>,
    pub disabled_when_path: Option<LitStr>,
    pub disabled_when_values: Option<Vec<JsonScalar>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Widget {
    Text,
    Number,
    Select,
    Textarea,
    KeyValue,
    Switch,
    JsonEditor,
}

pub fn parse_container(attrs: &[Attribute]) -> syn::Result<ContainerAttrs> {
    let mut result = ContainerAttrs::default();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("task_params")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("schema_version") {
                return parse_once(&mut result.schema_version, &meta, "schema_version");
            }
            if meta.path.is_ident("validate_with") {
                return parse_once(&mut result.validate_with, &meta, "validate_with");
            }
            if meta.path.is_ident("render_with") {
                return parse_once(&mut result.render_with, &meta, "render_with");
            }
            Err(meta.error("unsupported task_params argument"))
        })?;
    }
    Ok(result)
}

pub fn parse_field(attrs: &[Attribute]) -> syn::Result<FieldAttrs> {
    let mut result = FieldAttrs::default();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("param_field")) {
        attr.parse_nested_meta(|meta| parse_field_meta(meta, &mut result))?;
    }
    Ok(result)
}

pub fn required_schema_version(attrs: &ContainerAttrs) -> syn::Result<&Expr> {
    attrs
        .schema_version
        .as_ref()
        .ok_or_else(|| syn::Error::new_spanned(TokenStream2::new(), "task_params schema_version is required"))
}

pub fn label_key(field_name: &str, attrs: &FieldAttrs) -> LitStr {
    attrs
        .label_key
        .clone()
        .unwrap_or_else(|| LitStr::new(&format!("scheduler.param_fields.{field_name}"), proc_macro2::Span::call_site()))
}

impl Widget {
    fn parse(value: LitStr) -> syn::Result<Self> {
        match value.value().as_str() {
            "text" => Ok(Self::Text),
            "number" => Ok(Self::Number),
            "select" => Ok(Self::Select),
            "textarea" => Ok(Self::Textarea),
            "key_value" => Ok(Self::KeyValue),
            "switch" => Ok(Self::Switch),
            "json_editor" => Ok(Self::JsonEditor),
            _ => Err(syn::Error::new_spanned(value, "unsupported task parameter widget")),
        }
    }

    pub fn from_default(value: &str) -> Self {
        match value {
            "text" => Self::Text,
            "number" => Self::Number,
            "switch" => Self::Switch,
            "key_value" => Self::KeyValue,
            "json_editor" => Self::JsonEditor,
            _ => unreachable!("macro default widgets are exhaustive"),
        }
    }

    pub fn tokens(self) -> TokenStream2 {
        let variant = match self {
            Self::Text => quote!(Text),
            Self::Number => quote!(Number),
            Self::Select => quote!(Select),
            Self::Textarea => quote!(Textarea),
            Self::KeyValue => quote!(KeyValue),
            Self::Switch => quote!(Switch),
            Self::JsonEditor => quote!(JsonEditor),
        };
        quote! { scheduler::domain::ParamWidget::#variant }
    }
}

fn parse_field_meta(meta: ParseNestedMeta<'_>, result: &mut FieldAttrs) -> syn::Result<()> {
    if parse_field_presentation(&meta, result)? || parse_field_validation(&meta, result)? {
        return Ok(());
    }
    Err(meta.error("unsupported param_field argument"))
}

fn parse_field_presentation(meta: &ParseNestedMeta<'_>, result: &mut FieldAttrs) -> syn::Result<bool> {
    if meta.path.is_ident("required") {
        if result.required {
            return Err(meta.error("duplicate param_field required"));
        }
        result.required = true;
        return Ok(true);
    }
    if meta.path.is_ident("widget") {
        let value: LitStr = parse_value(meta)?;
        set_once(&mut result.widget, Widget::parse(value)?, meta.error("duplicate macro argument: widget"))?;
        return Ok(true);
    }
    if meta.path.is_ident("label_key") {
        parse_once(&mut result.label_key, meta, "label_key")?;
        return Ok(true);
    }
    if meta.path.is_ident("placeholder_key") {
        parse_once(&mut result.placeholder_key, meta, "placeholder_key")?;
        return Ok(true);
    }
    if meta.path.is_ident("help_key") {
        parse_once(&mut result.help_key, meta, "help_key")?;
        return Ok(true);
    }
    Ok(false)
}

fn parse_field_validation(meta: &ParseNestedMeta<'_>, result: &mut FieldAttrs) -> syn::Result<bool> {
    if meta.path.is_ident("pattern") {
        parse_once(&mut result.pattern, meta, "pattern")?;
        return Ok(true);
    }
    if meta.path.is_ident("options") {
        let values = parse_lit_str_array(parse_value(meta)?)?;
        set_once(&mut result.options, values, meta.error("duplicate macro argument: options"))?;
        return Ok(true);
    }
    if meta.path.is_ident("default") {
        parse_once(&mut result.default, meta, "default")?;
        return Ok(true);
    }
    if meta.path.is_ident("disabled_when_path") {
        parse_once(&mut result.disabled_when_path, meta, "disabled_when_path")?;
        return Ok(true);
    }
    if meta.path.is_ident("disabled_when_values") {
        let values = JsonScalar::parse_array(parse_value(meta)?)?;
        set_once(
            &mut result.disabled_when_values,
            values,
            meta.error("duplicate macro argument: disabled_when_values"),
        )?;
        return Ok(true);
    }
    Ok(false)
}

fn parse_once<T: syn::parse::Parse>(slot: &mut Option<T>, meta: &ParseNestedMeta<'_>, name: &str) -> syn::Result<()> {
    let value = parse_value(meta)?;
    set_once(slot, value, meta.error(format!("duplicate macro argument: {name}")))
}

fn parse_value<T: syn::parse::Parse>(meta: &ParseNestedMeta<'_>) -> syn::Result<T> {
    meta.value()?.parse()
}

fn set_once<T>(slot: &mut Option<T>, value: T, duplicate_error: syn::Error) -> syn::Result<()> {
    if slot.is_some() {
        return Err(duplicate_error);
    }
    *slot = Some(value);
    Ok(())
}

fn parse_lit_str_array(array: ExprArray) -> syn::Result<Vec<LitStr>> {
    array
        .elems
        .into_iter()
        .map(|expr| match expr {
            Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) => Ok(value),
            other => Err(syn::Error::new_spanned(other, "expected string literal")),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_container, parse_field};

    #[test]
    fn schema_version_accepts_an_expression() {
        let input: syn::DeriveInput = syn::parse_quote!(
            #[task_params(schema_version = VERSION)]
            struct Params {}
        );
        assert!(parse_container(&input.attrs).expect("attributes should parse").schema_version.is_some());
    }

    #[test]
    fn duplicate_and_unknown_arguments_fail() {
        let duplicate: syn::Field = syn::parse_quote!(#[param_field(widget = "text", widget = "text")] value: String);
        assert!(parse_field(&duplicate.attrs).is_err());
        let unknown: syn::Field = syn::parse_quote!(#[param_field(unknown = "value")] value: String);
        assert!(parse_field(&unknown.attrs).is_err());
    }
}
