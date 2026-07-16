use config_rs::{Value, ValueKind};
use serde::de::Visitor;

use super::{
    Input, InterpolatingDeserializer, InterpolationError,
    access::{visit_mapping, visit_sequence},
};

macro_rules! deserialize_number {
    ($method:ident, $visit:ident, $type:ty, $expected:literal) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            match self.resolve()?.input {
                Input::Environment { variable, value } => {
                    let parsed = value
                        .parse::<$type>()
                        .map_err(|_| InterpolationError::invalid_environment_value(variable, $expected))?;
                    visitor.$visit(parsed)
                }
                Input::Config(value) => serde::Deserializer::$method(value, visitor).map_err(Into::into),
            }
        }
    };
}

impl<'de> serde::Deserializer<'de> for InterpolatingDeserializer<'_> {
    type Error = InterpolationError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let environment = self.environment;
        match self.resolve()?.input {
            Input::Environment { value, .. } => visitor.visit_string(value),
            Input::Config(value) => deserialize_config_any(value, environment, visitor),
        }
    }

    deserialize_number!(deserialize_bool, visit_bool, bool, "bool");
    deserialize_number!(deserialize_i8, visit_i8, i8, "i8");
    deserialize_number!(deserialize_i16, visit_i16, i16, "i16");
    deserialize_number!(deserialize_i32, visit_i32, i32, "i32");
    deserialize_number!(deserialize_i64, visit_i64, i64, "i64");
    deserialize_number!(deserialize_i128, visit_i128, i128, "i128");
    deserialize_number!(deserialize_u8, visit_u8, u8, "u8");
    deserialize_number!(deserialize_u16, visit_u16, u16, "u16");
    deserialize_number!(deserialize_u32, visit_u32, u32, "u32");
    deserialize_number!(deserialize_u64, visit_u64, u64, "u64");
    deserialize_number!(deserialize_u128, visit_u128, u128, "u128");
    deserialize_number!(deserialize_f32, visit_f32, f32, "f32");
    deserialize_number!(deserialize_f64, visit_f64, f64, "f64");
    deserialize_number!(deserialize_char, visit_char, char, "char");

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.resolve()?.input {
            Input::Environment { value, .. } => visitor.visit_string(value),
            Input::Config(value) => serde::Deserializer::deserialize_string(value, visitor).map_err(Into::into),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.resolve()?.input {
            Input::Environment { value, .. } => visitor.visit_byte_buf(value.into_bytes()),
            Input::Config(value) => serde::Deserializer::deserialize_bytes(value, visitor).map_err(Into::into),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let resolved = self.resolve()?;
        match &resolved.input {
            Input::Environment { value, .. } if value.is_empty() => visitor.visit_none(),
            Input::Config(value) if matches!(&value.kind, ValueKind::Nil) => visitor.visit_none(),
            _ => visitor.visit_some(resolved),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.resolve()?.input {
            Input::Environment { variable, .. } => Err(InterpolationError::environment_type(variable, "unit")),
            Input::Config(value) => serde::Deserializer::deserialize_unit(value, visitor).map_err(Into::into),
        }
    }

    fn deserialize_unit_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let environment = self.environment;
        match self.resolve()?.input {
            Input::Environment { variable, .. } => Err(InterpolationError::environment_type(variable, "sequence")),
            Input::Config(value) => deserialize_config_sequence(value, environment, visitor),
        }
    }

    fn deserialize_tuple<V>(self, _: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(self, _: &'static str, _: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        deserialize_mapping(self, visitor)
    }

    fn deserialize_struct<V>(self, _: &'static str, _: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        deserialize_mapping(self, visitor)
    }

    fn deserialize_enum<V>(self, name: &'static str, variants: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.resolve()?.input {
            Input::Environment { variable, .. } => Err(InterpolationError::environment_type(variable, "enum")),
            Input::Config(value) => serde::Deserializer::deserialize_enum(value, name, variants, visitor).map_err(Into::into),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

fn deserialize_config_any<'de, V>(value: Value, environment: &dyn crate::EnvironmentReader, visitor: V) -> Result<V::Value, InterpolationError>
where
    V: Visitor<'de>,
{
    match value.kind {
        ValueKind::Array(values) => visit_sequence(values, environment, visitor),
        ValueKind::Table(values) => visit_mapping(values.into_iter().collect(), environment, visitor),
        kind => serde::Deserializer::deserialize_any(Value::new(None, kind), visitor).map_err(Into::into),
    }
}

fn deserialize_config_sequence<'de, V>(value: Value, environment: &dyn crate::EnvironmentReader, visitor: V) -> Result<V::Value, InterpolationError>
where
    V: Visitor<'de>,
{
    match value.kind {
        ValueKind::Array(values) => visit_sequence(values, environment, visitor),
        kind => serde::Deserializer::deserialize_seq(Value::new(None, kind), visitor).map_err(Into::into),
    }
}

fn deserialize_mapping<'de, V>(deserializer: InterpolatingDeserializer<'_>, visitor: V) -> Result<V::Value, InterpolationError>
where
    V: Visitor<'de>,
{
    let environment = deserializer.environment;
    match deserializer.resolve()?.input {
        Input::Environment { variable, .. } => Err(InterpolationError::environment_type(variable, "mapping")),
        Input::Config(value) => match value.kind {
            ValueKind::Table(values) => visit_mapping(values.into_iter().collect(), environment, visitor),
            kind => serde::Deserializer::deserialize_map(Value::new(None, kind), visitor).map_err(Into::into),
        },
    }
}
