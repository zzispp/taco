use quote::quote;
use syn::{GenericArgument, PathArguments, Type, TypePath};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchemaType {
    String,
    Number,
    Boolean,
    StringMap,
    Array(Box<SchemaType>),
}

#[derive(Clone, Debug)]
pub struct FieldType {
    pub optional: bool,
    pub schema: SchemaType,
}

impl FieldType {
    pub fn parse(ty: &Type) -> syn::Result<Self> {
        if let Some(inner) = generic_inner(ty, "Option")? {
            return Ok(Self {
                optional: true,
                schema: SchemaType::parse(inner, false)?,
            });
        }
        Ok(Self {
            optional: false,
            schema: SchemaType::parse(ty, false)?,
        })
    }
}

impl SchemaType {
    fn parse(ty: &Type, inside_collection: bool) -> syn::Result<Self> {
        if outer_ident(ty).as_deref() == Some("Option") {
            let context = if inside_collection { " inside a collection" } else { "" };
            return Err(syn::Error::new_spanned(
                ty,
                format!("Option is supported only as the outer field type{context}"),
            ));
        }
        if is_plain_type(ty, "String") {
            return Ok(Self::String);
        }
        if is_plain_type(ty, "bool") {
            return Ok(Self::Boolean);
        }
        if is_number_type(ty) {
            return Ok(Self::Number);
        }
        if matches!(outer_ident(ty).as_deref(), Some("BTreeMap") | Some("HashMap")) {
            return parse_string_map(ty);
        }
        if outer_ident(ty).as_deref() == Some("Vec") {
            let inner = required_single_type_argument(ty, "Vec")?;
            return Ok(Self::Array(Box::new(Self::parse(inner, true)?)));
        }
        Err(syn::Error::new_spanned(
            ty,
            "unsupported task parameter type; expected String, bool, a numeric type, Option<T>, Vec<T>, or a String map",
        ))
    }

    pub fn schema_tokens(&self, pattern: Option<&syn::LitStr>, options: &[syn::LitStr]) -> proc_macro2::TokenStream {
        match self {
            Self::String => string_schema(pattern, options),
            Self::Number => quote! {
                scheduler::domain::ParamSchema::Number(scheduler::domain::NumberParamSchema::default())
            },
            Self::Boolean => quote! {
                scheduler::domain::ParamSchema::Boolean(scheduler::domain::BooleanParamSchema::default())
            },
            Self::StringMap => string_map_schema(),
            Self::Array(inner) => {
                let items = inner.schema_tokens(None, &[]);
                quote! {
                    scheduler::domain::ParamSchema::Array(scheduler::domain::ArrayParamSchema {
                        items: Box::new(#items),
                    })
                }
            }
        }
    }

    pub fn default_widget(&self) -> &'static str {
        match self {
            Self::String => "text",
            Self::Number => "number",
            Self::Boolean => "switch",
            Self::StringMap => "key_value",
            Self::Array(_) => "json_editor",
        }
    }

    pub fn accepts_string_rules(&self) -> bool {
        matches!(self, Self::String)
    }
}

fn parse_string_map(ty: &Type) -> syn::Result<SchemaType> {
    let args = generic_type_arguments(ty)?;
    if args.len() == 2 && is_plain_type(args[0], "String") && is_plain_type(args[1], "String") {
        return Ok(SchemaType::StringMap);
    }
    Err(syn::Error::new_spanned(ty, "task parameter maps must use String keys and String values"))
}

fn string_schema(pattern: Option<&syn::LitStr>, options: &[syn::LitStr]) -> proc_macro2::TokenStream {
    let pattern = match pattern {
        Some(value) => quote! { Some(#value.into()) },
        None => quote! { None },
    };
    quote! {
        scheduler::domain::ParamSchema::String(scheduler::domain::StringParamSchema {
            format: None,
            pattern: #pattern,
            enum_values: vec![#(#options.into()),*],
        })
    }
}

fn string_map_schema() -> proc_macro2::TokenStream {
    let string = string_schema(None, &[]);
    quote! {
        scheduler::domain::ParamSchema::Record(scheduler::domain::RecordParamSchema {
            key: Box::new(#string),
            value: Box::new(#string),
        })
    }
}

fn generic_inner<'a>(ty: &'a Type, outer: &str) -> syn::Result<Option<&'a Type>> {
    if outer_ident(ty).as_deref() != Some(outer) {
        return Ok(None);
    }
    required_single_type_argument(ty, outer).map(Some)
}

fn required_single_type_argument<'a>(ty: &'a Type, outer: &str) -> syn::Result<&'a Type> {
    let args = generic_type_arguments(ty)?;
    if args.len() != 1 {
        return Err(syn::Error::new_spanned(ty, format!("{outer} requires exactly one type argument")));
    }
    Ok(args[0])
}

fn generic_type_arguments(ty: &Type) -> syn::Result<Vec<&Type>> {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return Err(syn::Error::new_spanned(ty, "task parameter types must be owned path types"));
    };
    let Some(segment) = path.segments.last() else {
        return Err(syn::Error::new_spanned(ty, "task parameter type path cannot be empty"));
    };
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(syn::Error::new_spanned(ty, "expected generic type arguments"));
    };
    arguments
        .args
        .iter()
        .map(|argument| match argument {
            GenericArgument::Type(value) => Ok(value),
            other => Err(syn::Error::new_spanned(other, "only type arguments are supported")),
        })
        .collect()
}

fn is_number_type(ty: &Type) -> bool {
    const NUMBER_TYPES: [&str; 14] = [
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64",
    ];
    NUMBER_TYPES.iter().any(|name| is_plain_type(ty, name))
}

fn is_plain_type(ty: &Type, expected: &str) -> bool {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return false;
    };
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == expected && matches!(segment.arguments, PathArguments::None))
}

fn outer_ident(ty: &Type) -> Option<String> {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return None;
    };
    path.segments.last().map(|segment| segment.ident.to_string())
}

#[cfg(test)]
mod tests {
    use super::{FieldType, SchemaType};

    #[test]
    fn parses_recursive_arrays_and_outer_option() {
        let parsed = FieldType::parse(&syn::parse_quote!(Option<Vec<Vec<u32>>>)).expect("type should parse");
        assert!(parsed.optional);
        assert_eq!(parsed.schema, SchemaType::Array(Box::new(SchemaType::Array(Box::new(SchemaType::Number)))));
    }

    #[test]
    fn rejects_nested_option_and_non_string_map() {
        assert!(FieldType::parse(&syn::parse_quote!(Vec<Option<String>>)).is_err());
        assert!(FieldType::parse(&syn::parse_quote!(std::collections::HashMap<String, u32>)).is_err());
    }

    #[test]
    fn rejects_unknown_structures() {
        assert!(FieldType::parse(&syn::parse_quote!(CustomParams)).is_err());
        assert!(FieldType::parse(&syn::parse_quote!((String, String))).is_err());
        assert!(FieldType::parse(&syn::parse_quote!(&str)).is_err());
    }
}
