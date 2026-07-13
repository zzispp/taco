use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParamSchema {
    Object(ObjectParamSchema),
    String(StringParamSchema),
    Number(NumberParamSchema),
    Boolean(BooleanParamSchema),
    Record(RecordParamSchema),
    Array(ArrayParamSchema),
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ObjectParamSchema {
    pub properties: BTreeMap<String, ParamSchema>,
    pub required: Vec<String>,
    pub additional_properties: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct StringParamSchema {
    pub format: Option<String>,
    pub pattern: Option<String>,
    pub enum_values: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct NumberParamSchema {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct BooleanParamSchema {}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RecordParamSchema {
    pub key: Box<ParamSchema>,
    pub value: Box<ParamSchema>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ArrayParamSchema {
    pub items: Box<ParamSchema>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamWidget {
    Text,
    Number,
    Select,
    Textarea,
    KeyValue,
    Switch,
    JsonEditor,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ParamCondition {
    pub path: String,
    pub values: Vec<Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ParamFieldSpec {
    pub path: String,
    pub label_key: String,
    pub widget: ParamWidget,
    pub placeholder_key: Option<String>,
    pub help_key: Option<String>,
    pub options: Vec<String>,
    pub disabled_when: Option<ParamCondition>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ParamUiSpec {
    pub fields: Vec<ParamFieldSpec>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TaskParamFormSpec {
    pub schema_version: i16,
    pub schema: ParamSchema,
    pub ui: ParamUiSpec,
}
