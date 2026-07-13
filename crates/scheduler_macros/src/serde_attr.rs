use syn::{Attribute, Expr, ExprLit, ExprPath, Lit, Meta, Token, punctuated::Punctuated};

#[derive(Clone)]
pub enum SerdeDefault {
    DefaultTrait,
    Function(ExprPath),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenameRule {
    Lower,
    Upper,
    Pascal,
    Camel,
    Snake,
    ScreamingSnake,
    Kebab,
    ScreamingKebab,
}

#[derive(Clone, Default)]
pub struct ContainerSerde {
    pub default: Option<SerdeDefault>,
    pub rename_all: Option<RenameRule>,
}

#[derive(Clone, Default)]
pub struct FieldSerde {
    pub default: Option<SerdeDefault>,
    pub rename: Option<String>,
}

pub fn parse_container(attrs: &[Attribute]) -> syn::Result<ContainerSerde> {
    let mut result = ContainerSerde::default();
    let mut deny_unknown_fields = false;
    for meta in serde_meta(attrs)? {
        if meta.path().is_ident("default") {
            set_once(
                &mut result.default,
                parse_default(&meta)?,
                syn::Error::new_spanned(&meta, "duplicate serde default"),
            )?;
        } else if meta.path().is_ident("rename_all") {
            let rename = parse_directional_name(&meta, "rename_all")?;
            let rule = RenameRule::parse(&rename, &meta)?;
            set_once(&mut result.rename_all, rule, syn::Error::new_spanned(&meta, "duplicate serde rename_all"))?;
        } else if meta.path().is_ident("deny_unknown_fields") {
            parse_marker(&mut deny_unknown_fields, &meta, "deny_unknown_fields")?;
        } else {
            return Err(incompatible_container(&meta));
        }
    }
    Ok(result)
}

pub fn parse_field(attrs: &[Attribute]) -> syn::Result<FieldSerde> {
    let mut result = FieldSerde::default();
    for meta in serde_meta(attrs)? {
        if meta.path().is_ident("default") {
            set_once(
                &mut result.default,
                parse_default(&meta)?,
                syn::Error::new_spanned(&meta, "duplicate serde default"),
            )?;
        } else if meta.path().is_ident("rename") {
            let rename = parse_directional_name(&meta, "rename")?;
            set_once(&mut result.rename, rename, syn::Error::new_spanned(&meta, "duplicate serde rename"))?;
        } else {
            return Err(incompatible_field(&meta));
        }
    }
    Ok(result)
}

impl RenameRule {
    fn parse(value: &str, meta: &Meta) -> syn::Result<Self> {
        let rule = match value {
            "lowercase" => Self::Lower,
            "UPPERCASE" => Self::Upper,
            "PascalCase" => Self::Pascal,
            "camelCase" => Self::Camel,
            "snake_case" => Self::Snake,
            "SCREAMING_SNAKE_CASE" => Self::ScreamingSnake,
            "kebab-case" => Self::Kebab,
            "SCREAMING-KEBAB-CASE" => Self::ScreamingKebab,
            _ => return Err(syn::Error::new_spanned(meta, "unsupported serde rename_all rule")),
        };
        Ok(rule)
    }

    pub fn apply(self, field: &str) -> String {
        match self {
            Self::Lower | Self::Snake => field.to_owned(),
            Self::Upper | Self::ScreamingSnake => field.to_ascii_uppercase(),
            Self::Pascal => pascal_case(field),
            Self::Camel => camel_case(field),
            Self::Kebab => field.replace('_', "-"),
            Self::ScreamingKebab => field.to_ascii_uppercase().replace('_', "-"),
        }
    }
}

fn serde_meta(attrs: &[Attribute]) -> syn::Result<Vec<Meta>> {
    let mut values = Vec::new();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
        let parsed = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        values.extend(parsed);
    }
    Ok(values)
}

fn parse_default(meta: &Meta) -> syn::Result<SerdeDefault> {
    match meta {
        Meta::Path(_) => Ok(SerdeDefault::DefaultTrait),
        Meta::NameValue(value) => {
            let literal = string_literal(&value.value, "serde default must be a function path string")?;
            Ok(SerdeDefault::Function(literal.parse()?))
        }
        Meta::List(_) => Err(syn::Error::new_spanned(meta, "serde default does not accept nested arguments")),
    }
}

fn parse_directional_name(meta: &Meta, name: &str) -> syn::Result<String> {
    match meta {
        Meta::NameValue(value) => Ok(string_literal(&value.value, &format!("serde {name} must be a string"))?.value()),
        Meta::List(list) => {
            let nested = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            let mut serialize = None;
            let mut deserialize = None;
            for item in nested {
                let Meta::NameValue(value) = &item else {
                    return Err(syn::Error::new_spanned(item, format!("serde {name} direction requires a string value")));
                };
                let literal = string_literal(&value.value, &format!("serde {name} direction must be a string"))?;
                if value.path.is_ident("serialize") {
                    set_once(
                        &mut serialize,
                        literal.value(),
                        syn::Error::new_spanned(&item, "duplicate serde serialize name"),
                    )?;
                } else if value.path.is_ident("deserialize") {
                    set_once(
                        &mut deserialize,
                        literal.value(),
                        syn::Error::new_spanned(&item, "duplicate serde deserialize name"),
                    )?;
                } else {
                    return Err(syn::Error::new_spanned(item, format!("unsupported serde {name} direction")));
                }
            }
            match (serialize, deserialize) {
                (Some(left), Some(right)) if left == right => Ok(left),
                _ => Err(syn::Error::new_spanned(
                    meta,
                    format!("serde {name} must use the same serialize and deserialize name"),
                )),
            }
        }
        Meta::Path(_) => Err(syn::Error::new_spanned(meta, format!("serde {name} requires a value"))),
    }
}

fn parse_marker(slot: &mut bool, meta: &Meta, name: &str) -> syn::Result<()> {
    if !matches!(meta, Meta::Path(_)) {
        return Err(syn::Error::new_spanned(meta, format!("serde {name} does not accept a value")));
    }
    if *slot {
        return Err(syn::Error::new_spanned(meta, format!("duplicate serde {name}")));
    }
    *slot = true;
    Ok(())
}

fn incompatible_container(meta: &Meta) -> syn::Error {
    syn::Error::new_spanned(meta, "this serde container attribute is incompatible with ScheduledTaskParams")
}

fn incompatible_field(meta: &Meta) -> syn::Error {
    syn::Error::new_spanned(meta, "this serde attribute is incompatible with ScheduledTaskParams")
}

fn string_literal<'a>(expr: &'a Expr, message: &str) -> syn::Result<&'a syn::LitStr> {
    match expr {
        Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) => Ok(value),
        _ => Err(syn::Error::new_spanned(expr, message)),
    }
}

fn set_once<T>(slot: &mut Option<T>, value: T, duplicate_error: syn::Error) -> syn::Result<()> {
    if slot.is_some() {
        return Err(duplicate_error);
    }
    *slot = Some(value);
    Ok(())
}

fn pascal_case(field: &str) -> String {
    field
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            characters
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + characters.as_str())
                .unwrap_or_default()
        })
        .collect()
}

fn camel_case(field: &str) -> String {
    let pascal = pascal_case(field);
    let mut characters = pascal.chars();
    characters
        .next()
        .map(|first| first.to_lowercase().collect::<String>() + characters.as_str())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{RenameRule, parse_container, parse_field};

    #[test]
    fn applies_supported_rename_rules() {
        assert_eq!(RenameRule::Camel.apply("request_body"), "requestBody");
        assert_eq!(RenameRule::Pascal.apply("request_body"), "RequestBody");
        assert_eq!(RenameRule::Kebab.apply("request_body"), "request-body");
    }

    #[test]
    fn accepts_equal_directional_rename_and_rejects_mismatch() {
        let equal: syn::Field = syn::parse_quote!(#[serde(rename(serialize = "wire", deserialize = "wire"))] value: String);
        assert_eq!(parse_field(&equal.attrs).expect("rename should parse").rename.as_deref(), Some("wire"));

        let mismatch: syn::Field = syn::parse_quote!(#[serde(rename(serialize = "out", deserialize = "in"))] value: String);
        assert!(parse_field(&mismatch.attrs).is_err());
    }

    #[test]
    fn rejects_shape_changing_attributes() {
        let fields: Vec<syn::Field> = vec![
            syn::parse_quote!(#[serde(flatten)] value: String),
            syn::parse_quote!(#[serde(skip)] value: String),
            syn::parse_quote!(#[serde(alias = "old")] value: String),
            syn::parse_quote!(#[serde(borrow)] value: String),
        ];
        for field in fields {
            assert!(parse_field(&field.attrs).is_err());
        }
    }

    #[test]
    fn container_uses_a_strict_attribute_allowlist() {
        let accepted: syn::DeriveInput = syn::parse_quote! {
            #[serde(default, rename_all = "camelCase", deny_unknown_fields)]
            struct Params { value: String }
        };
        assert!(parse_container(&accepted.attrs).is_ok());

        let rejected = [
            r#"#[serde(tag = "kind")] struct Params { value: String }"#,
            r#"#[serde(content = "value")] struct Params { value: String }"#,
            r#"#[serde(untagged)] struct Params { value: String }"#,
            r#"#[serde(remote = "Self")] struct Params { value: String }"#,
            r#"#[serde(unknown)] struct Params { value: String }"#,
        ];
        for source in rejected {
            let input = syn::parse_str::<syn::DeriveInput>(source).expect("test input must parse");
            assert!(parse_container(&input.attrs).is_err());
        }
    }
}
