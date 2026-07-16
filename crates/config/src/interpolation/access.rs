use std::vec;

use config_rs::Value;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};

use crate::EnvironmentReader;

use super::{InterpolatingDeserializer, InterpolationError};

pub(super) fn visit_sequence<'de, V>(values: Vec<Value>, environment: &dyn EnvironmentReader, visitor: V) -> Result<V::Value, InterpolationError>
where
    V: Visitor<'de>,
{
    visitor.visit_seq(InterpolatingSeqAccess {
        values: values.into_iter(),
        environment,
    })
}

pub(super) fn visit_mapping<'de, V>(values: Vec<(String, Value)>, environment: &dyn EnvironmentReader, visitor: V) -> Result<V::Value, InterpolationError>
where
    V: Visitor<'de>,
{
    visitor.visit_map(InterpolatingMapAccess {
        values: values.into_iter(),
        pending: None,
        environment,
    })
}

struct InterpolatingSeqAccess<'a> {
    values: vec::IntoIter<Value>,
    environment: &'a dyn EnvironmentReader,
}

impl<'de> SeqAccess<'de> for InterpolatingSeqAccess<'_> {
    type Error = InterpolationError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.values
            .next()
            .map(|value| seed.deserialize(InterpolatingDeserializer::new(value, self.environment)))
            .transpose()
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.values.len())
    }
}

struct InterpolatingMapAccess<'a> {
    values: vec::IntoIter<(String, Value)>,
    pending: Option<Value>,
    environment: &'a dyn EnvironmentReader,
}

impl<'de> MapAccess<'de> for InterpolatingMapAccess<'_> {
    type Error = InterpolationError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let Some((key, value)) = self.values.next() else {
            return Ok(None);
        };
        self.pending = Some(value);
        seed.deserialize(de::value::StringDeserializer::<InterpolationError>::new(key)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .pending
            .take()
            .ok_or_else(|| <InterpolationError as de::Error>::custom("mapping value requested before key"))?;
        seed.deserialize(InterpolatingDeserializer::new(value, self.environment))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.values.len())
    }
}
