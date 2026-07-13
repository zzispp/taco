use std::collections::{HashMap, HashSet};

use syn::{DeriveInput, Field, Ident, LitStr, Type};

use crate::{
    json_scalar::JsonScalarKind,
    serde_attr::{ContainerSerde, FieldSerde, SerdeDefault},
    task_params_attr::{ContainerAttrs, FieldAttrs, Widget, parse_container, parse_field},
    task_params_type::{FieldType, SchemaType},
};

pub struct ParamsModel {
    pub attrs: ContainerAttrs,
    pub serde: ContainerSerde,
    pub fields: Vec<FieldModel>,
}

pub struct FieldModel {
    pub ident: Ident,
    pub rust_name: String,
    pub wire_name: String,
    pub ty: Type,
    pub attrs: FieldAttrs,
    pub serde: FieldSerde,
    pub field_type: FieldType,
    pub widget: Widget,
    pub disabled_wire_path: Option<LitStr>,
}

#[derive(Clone)]
struct DisabledTarget {
    wire_name: String,
    schema: SchemaType,
}

impl ParamsModel {
    pub fn parse(input: &DeriveInput, fields: Vec<Field>) -> syn::Result<Self> {
        let attrs = parse_container(&input.attrs)?;
        let serde = crate::serde_attr::parse_container(&input.attrs)?;
        let mut fields = fields.iter().map(|field| FieldModel::parse(field, &serde)).collect::<syn::Result<Vec<_>>>()?;
        validate_unique_wire_names(&fields)?;
        resolve_disabled_paths(&mut fields)?;
        Ok(Self { attrs, serde, fields })
    }
}

impl FieldModel {
    fn parse(field: &Field, container: &ContainerSerde) -> syn::Result<Self> {
        let ident = field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new_spanned(field, "ScheduledTaskParams requires named fields"))?;
        let rust_name = ident.to_string().trim_start_matches("r#").to_owned();
        let attrs = parse_field(&field.attrs)?;
        let serde = crate::serde_attr::parse_field(&field.attrs)?;
        let field_type = FieldType::parse(&field.ty)?;
        validate_string_rules(&attrs, &field_type.schema, field)?;
        let widget = attrs.widget.unwrap_or_else(|| Widget::from_default(field_type.schema.default_widget()));
        validate_widget(widget, &field_type.schema, field)?;
        validate_disabled_attrs(&attrs, field)?;
        let wire_name = serde
            .rename
            .clone()
            .or_else(|| container.rename_all.map(|rule| rule.apply(&rust_name)))
            .unwrap_or_else(|| rust_name.clone());
        Ok(Self {
            ident,
            rust_name,
            wire_name,
            ty: field.ty.clone(),
            attrs,
            serde,
            field_type,
            widget,
            disabled_wire_path: None,
        })
    }

    pub fn required(&self, container_default: Option<&SerdeDefault>) -> bool {
        self.attrs.required || (!self.field_type.optional && self.serde.default.is_none() && container_default.is_none())
    }
}

fn validate_unique_wire_names(fields: &[FieldModel]) -> syn::Result<()> {
    let mut names = HashSet::new();
    for field in fields {
        if !names.insert(field.wire_name.as_str()) {
            return Err(syn::Error::new_spanned(
                &field.ident,
                format!("duplicate serialized task parameter name: {}", field.wire_name),
            ));
        }
    }
    Ok(())
}

fn resolve_disabled_paths(fields: &mut [FieldModel]) -> syn::Result<()> {
    let rust_names = fields
        .iter()
        .map(|field| (field.rust_name.clone(), DisabledTarget::from(field)))
        .collect::<HashMap<_, _>>();
    let wire_names = fields
        .iter()
        .map(|field| (field.wire_name.clone(), DisabledTarget::from(field)))
        .collect::<HashMap<_, _>>();
    for field in fields {
        let Some(path) = field.attrs.disabled_when_path.as_ref() else {
            continue;
        };
        let target = resolve_disabled_target(path, &rust_names, &wire_names)?;
        validate_disabled_values(field, target)?;
        field.disabled_wire_path = Some(LitStr::new(&target.wire_name, path.span()));
    }
    Ok(())
}

impl From<&FieldModel> for DisabledTarget {
    fn from(field: &FieldModel) -> Self {
        Self {
            wire_name: field.wire_name.clone(),
            schema: field.field_type.schema.clone(),
        }
    }
}

fn resolve_disabled_target<'a>(
    path: &LitStr,
    rust_names: &'a HashMap<String, DisabledTarget>,
    wire_names: &'a HashMap<String, DisabledTarget>,
) -> syn::Result<&'a DisabledTarget> {
    let input = path.value();
    rust_names
        .get(&input)
        .or_else(|| wire_names.get(&input))
        .ok_or_else(|| syn::Error::new_spanned(path, "disabled_when_path must reference a field in this parameter struct"))
}

fn validate_disabled_values(field: &FieldModel, target: &DisabledTarget) -> syn::Result<()> {
    let Some(expected) = scalar_kind(&target.schema) else {
        return Err(syn::Error::new_spanned(
            field.attrs.disabled_when_path.as_ref().expect("validated condition path"),
            "disabled_when_path must reference a String, bool, or numeric field",
        ));
    };
    let values = field.attrs.disabled_when_values.as_deref().expect("validated condition values");
    for value in values {
        if value.kind() != expected {
            return Err(syn::Error::new_spanned(
                value.expression(),
                "disabled_when_values must match the referenced field's JSON scalar type",
            ));
        }
    }
    Ok(())
}

const fn scalar_kind(schema: &SchemaType) -> Option<JsonScalarKind> {
    match schema {
        SchemaType::String => Some(JsonScalarKind::String),
        SchemaType::Number => Some(JsonScalarKind::Number),
        SchemaType::Boolean => Some(JsonScalarKind::Boolean),
        SchemaType::StringMap | SchemaType::Array(_) => None,
    }
}

fn validate_string_rules(attrs: &FieldAttrs, schema: &SchemaType, field: &Field) -> syn::Result<()> {
    if (attrs.pattern.is_some() || attrs.options.is_some()) && !schema.accepts_string_rules() {
        return Err(syn::Error::new_spanned(field, "pattern and options are supported only for String fields"));
    }
    Ok(())
}

fn validate_disabled_attrs(attrs: &FieldAttrs, field: &Field) -> syn::Result<()> {
    if attrs.disabled_when_path.is_some() != attrs.disabled_when_values.is_some() {
        return Err(syn::Error::new_spanned(
            field,
            "disabled_when_path and disabled_when_values must be specified together",
        ));
    }
    Ok(())
}

fn validate_widget(widget: Widget, schema: &SchemaType, field: &Field) -> syn::Result<()> {
    let valid = match schema {
        SchemaType::String => matches!(widget, Widget::Text | Widget::Select | Widget::Textarea),
        SchemaType::Number => matches!(widget, Widget::Number),
        SchemaType::Boolean => matches!(widget, Widget::Switch),
        SchemaType::StringMap => matches!(widget, Widget::KeyValue | Widget::JsonEditor),
        SchemaType::Array(_) => matches!(widget, Widget::JsonEditor),
    };
    if valid {
        return Ok(());
    }
    Err(syn::Error::new_spanned(field, "widget is incompatible with this task parameter type"))
}

#[cfg(test)]
mod tests {
    use super::ParamsModel;

    fn model(input: syn::DeriveInput) -> syn::Result<ParamsModel> {
        let syn::Data::Struct(data) = &input.data else {
            unreachable!();
        };
        let syn::Fields::Named(fields) = &data.fields else {
            unreachable!();
        };
        ParamsModel::parse(&input, fields.named.iter().cloned().collect())
    }

    #[test]
    fn maps_serde_names_and_disabled_path() {
        let input = syn::parse_quote! {
            #[task_params(schema_version = 1)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                enabled_flag: bool,
                #[param_field(disabled_when_path = "enabled_flag", disabled_when_values = [false])]
                display_name: String,
            }
        };
        let parsed = model(input).expect("model should parse");
        assert_eq!(parsed.fields[0].wire_name, "enabledFlag");
        assert_eq!(
            parsed.fields[1].disabled_wire_path.as_ref().map(LitStrExt::text).as_deref(),
            Some("enabledFlag")
        );
    }

    #[test]
    fn rejects_disabled_value_type_mismatch() {
        let input = syn::parse_quote! {
            #[task_params(schema_version = 1)]
            struct Params {
                enabled: bool,
                #[param_field(disabled_when_path = "enabled", disabled_when_values = ["false"])]
                display_name: String,
            }
        };

        assert!(model(input).is_err());
    }

    #[test]
    fn rejects_duplicate_wire_names_and_bad_widgets() {
        let duplicate = syn::parse_quote! {
            #[task_params(schema_version = 1)]
            struct Params { #[serde(rename = "value")] left: String, value: String }
        };
        assert!(model(duplicate).is_err());

        let widget = syn::parse_quote! {
            #[task_params(schema_version = 1)]
            struct Params { #[param_field(widget = "text")] count: u32 }
        };
        assert!(model(widget).is_err());
    }

    trait LitStrExt {
        fn text(&self) -> String;
    }

    impl LitStrExt for syn::LitStr {
        fn text(&self) -> String {
            self.value()
        }
    }
}
