use crate::{error::*, value::object::*};
use serde::{
    de::{
        DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor,
    },
    forward_to_deserialize_any, Deserialize,
};

pub fn deserialize<'a, T>(value: &'a Object) -> Result<T>
where
    T: Deserialize<'a>,
{
    T::deserialize(Deserializer::from_object(value))
}

#[derive(Debug)]
pub struct Deserializer<'de> {
    input: &'de Object,
}

impl<'de> Deserializer<'de> {
    pub fn from_object(input: &'de Object) -> Self {
        Self { input }
    }
}

impl<'de> serde::de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input {
            Object::Unit => visitor.visit_unit(),
            Object::Bool(v) => visitor.visit_bool(*v),
            Object::Number(v) => match v {
                Number::SignedInteger(v) => visitor.visit_i64(*v),
                Number::UnsignedInteger(v) => visitor.visit_u64(*v),
                Number::Float(v) => visitor.visit_f64(*v),
            },
            Object::String(v) => visitor.visit_borrowed_str(v),
            Object::Wrapper(v) => visitor.visit_newtype_struct(Self::from_object(v)),
            Object::Array(v) => visitor.visit_seq(SeqDeserializer {
                values: v.as_slice(),
                index: 0,
            }),
            Object::Map(v) => visitor.visit_map(MapDeserializer {
                values: v.as_slice(),
                index: 0,
            }),
            Object::Option(v) => match v {
                Some(v) => visitor.visit_some(Self::from_object(v)),
                None => visitor.visit_none(),
            },
            Object::Variant { name, value } => match &**value {
                Variant::Unit => visitor.visit_enum(EnumDeserializer::Unit { name }),
                Variant::Wrapper(v) => {
                    visitor.visit_enum(EnumDeserializer::NewType { name, content: v })
                }
                Variant::Array(v) => {
                    visitor.visit_enum(EnumDeserializer::Tuple { name, content: v })
                }
                Variant::Map(v) => {
                    visitor.visit_enum(EnumDeserializer::Struct { name, content: v })
                }
            },
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any newtype_struct enum
    }
}

#[derive(Debug)]
pub struct SeqDeserializer<'de> {
    values: &'de [Object],
    index: usize,
}

impl<'de> SeqAccess<'de> for SeqDeserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(value) = self.values.get(self.index) {
            self.index += 1;
            return seed.deserialize(Deserializer::from_object(value)).map(Some);
        }
        Ok(None)
    }
}

#[derive(Debug)]
pub struct MapDeserializer<'de> {
    values: &'de [(Object, Object)],
    index: usize,
}

impl<'de> MapAccess<'de> for MapDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, _)) = self.values.get(self.index) {
            return seed.deserialize(Deserializer::from_object(key)).map(Some);
        }
        Ok(None)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some((_, value)) = self.values.get(self.index) {
            self.index += 1;
            return seed.deserialize(Deserializer::from_object(value));
        }
        Err(Error::ExpectedMapEntry)
    }

    fn next_entry_seed<K, V>(&mut self, kseed: K, vseed: V) -> Result<Option<(K::Value, V::Value)>>
    where
        K: DeserializeSeed<'de>,
        V: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.values.get(self.index) {
            self.index += 1;
            let key = kseed.deserialize(Deserializer::from_object(key))?;
            let value = vseed.deserialize(Deserializer::from_object(value))?;
            return Ok(Some((key, value)));
        }
        Ok(None)
    }
}

#[derive(Debug)]
pub struct StructDeserializer<'de> {
    values: &'de [(String, Object)],
    index: usize,
}

impl<'de> MapAccess<'de> for StructDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, _)) = self.values.get(self.index) {
            return seed.deserialize(key.as_str().into_deserializer()).map(Some);
        }
        Ok(None)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some((_, value)) = self.values.get(self.index) {
            self.index += 1;
            return seed.deserialize(Deserializer::from_object(value));
        }
        Err(Error::ExpectedStructField)
    }

    fn next_entry_seed<K, V>(&mut self, kseed: K, vseed: V) -> Result<Option<(K::Value, V::Value)>>
    where
        K: DeserializeSeed<'de>,
        V: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.values.get(self.index) {
            self.index += 1;
            let key = kseed.deserialize(key.as_str().into_deserializer())?;
            let value = vseed.deserialize(Deserializer::from_object(value))?;
            return Ok(Some((key, value)));
        }
        Ok(None)
    }
}

#[derive(Debug)]
enum EnumDeserializer<'de> {
    Unit {
        name: &'de str,
    },
    NewType {
        name: &'de str,
        content: &'de Object,
    },
    Tuple {
        name: &'de str,
        content: &'de [Object],
    },
    Struct {
        name: &'de str,
        content: &'de [(String, Object)],
    },
}

impl<'de> EnumDeserializer<'de> {
    fn name(&self) -> &'de str {
        match self {
            Self::Unit { name }
            | Self::NewType { name, .. }
            | Self::Tuple { name, .. }
            | Self::Struct { name, .. } => name,
        }
    }
}

impl<'de> EnumAccess<'de> for EnumDeserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let name = seed.deserialize(self.name().into_deserializer())?;
        Ok((name, self))
    }
}

impl<'de> VariantAccess<'de> for EnumDeserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        if let EnumDeserializer::Unit { .. } = self {
            return Ok(());
        }
        Err(Error::ExpectedUnitVariant)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        if let EnumDeserializer::NewType { content, .. } = self {
            return seed.deserialize(Deserializer::from_object(content));
        }
        Err(Error::ExpectedNewTypeVariant)
    }

    fn tuple_variant<V>(self, _: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let EnumDeserializer::Tuple { content, .. } = self {
            return visitor.visit_seq(SeqDeserializer {
                values: content,
                index: 0,
            });
        }
        Err(Error::ExpectedNewTypeVariant)
    }

    fn struct_variant<V>(self, _: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let EnumDeserializer::Struct { content, .. } = self {
            return visitor.visit_map(StructDeserializer {
                values: content,
                index: 0,
            });
        }
        Err(Error::ExpectedStructVariant)
    }
}

#[derive(Copy, Clone)]
pub struct ObjectVisitor;

impl<'de> Visitor<'de> for ObjectVisitor {
    type Value = Object;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("object data representation")
    }

    fn visit_bool<E>(self, value: bool) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Bool(value))
    }

    fn visit_i8<E>(self, value: i8) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Number(Number::SignedInteger(value as _)))
    }

    fn visit_i16<E>(self, value: i16) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Number(Number::SignedInteger(value as _)))
    }

    fn visit_i32<E>(self, value: i32) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Number(Number::SignedInteger(value as _)))
    }

    fn visit_i64<E>(self, value: i64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Number(Number::SignedInteger(value)))
    }

    fn visit_u8<E>(self, value: u8) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Number(Number::UnsignedInteger(value as _)))
    }

    fn visit_u16<E>(self, value: u16) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Number(Number::UnsignedInteger(value as _)))
    }

    fn visit_u32<E>(self, value: u32) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Number(Number::UnsignedInteger(value as _)))
    }

    fn visit_u64<E>(self, value: u64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Number(Number::UnsignedInteger(value)))
    }

    fn visit_f32<E>(self, value: f32) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Number(Number::Float(value as _)))
    }

    fn visit_f64<E>(self, value: f64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Number(Number::Float(value)))
    }

    fn visit_char<E>(self, value: char) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::String(value.to_string()))
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::String(value.to_owned()))
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::String(value))
    }

    fn visit_bytes<E>(self, value: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Array(
            value
                .iter()
                .map(|value| Object::Number(Number::UnsignedInteger(*value as _)))
                .collect(),
        ))
    }

    fn visit_borrowed_bytes<E>(self, value: &'de [u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Array(
            value
                .iter()
                .map(|value| Object::Number(Number::UnsignedInteger(*value as _)))
                .collect(),
        ))
    }

    fn visit_byte_buf<E>(self, value: Vec<u8>) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Array(
            value
                .into_iter()
                .map(|value| Object::Number(Number::UnsignedInteger(value as _)))
                .collect(),
        ))
    }

    fn visit_none<E>(self) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Option(None))
    }

    fn visit_some<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Object::Option(Some(Box::new(
            deserializer.deserialize_any(ObjectVisitor)?,
        ))))
    }

    fn visit_unit<E>(self) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Object::Unit)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Object::Wrapper(Box::new(
            deserializer.deserialize_any(ObjectVisitor)?,
        )))
    }

    fn visit_seq<A>(self, mut access: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result = Vec::with_capacity(access.size_hint().unwrap_or_default());
        while let Some(v) = access.next_element()? {
            result.push(v);
        }
        Ok(Object::Array(result))
    }

    fn visit_map<A>(self, mut access: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut result = Vec::with_capacity(access.size_hint().unwrap_or_default());
        while let Some((k, v)) = access.next_entry()? {
            result.push((k, v));
        }
        Ok(Object::Map(result))
    }

    // TODO: what do we do with this? this obviously can be called, but at least JSON tests don't
    // do it, have to ask smart ppl what should i do to make it work, since neither serde docs nor
    // book shows how this works for self-describing types.
    // fn visit_enum<A>(self, mut access: A) -> std::result::Result<Self::Value, A::Error>
    // where
    //     A: EnumAccess<'de>,
    // {
    //     Err(serde::de::Error::invalid_type(Unexpected::Enum, &self))
    // }
}
