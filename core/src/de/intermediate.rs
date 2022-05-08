use crate::{error::*, value::intermediate::*};
use serde::{
    de::{
        DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor,
    },
    forward_to_deserialize_any, Deserialize,
};

pub fn deserialize<'a, T>(value: &'a Intermediate) -> Result<T>
where
    T: Deserialize<'a>,
{
    T::deserialize(Deserializer::from_intermediate(value))
}

pub struct Deserializer<'de> {
    input: &'de Intermediate,
}

impl<'de> Deserializer<'de> {
    pub fn from_intermediate(input: &'de Intermediate) -> Self {
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
            Intermediate::Unit => visitor.visit_unit(),
            Intermediate::Bool(v) => visitor.visit_bool(*v),
            Intermediate::I8(v) => visitor.visit_i8(*v),
            Intermediate::I16(v) => visitor.visit_i16(*v),
            Intermediate::I32(v) => visitor.visit_i32(*v),
            Intermediate::I64(v) => visitor.visit_i64(*v),
            Intermediate::I128(v) => visitor.visit_i128(*v),
            Intermediate::U8(v) => visitor.visit_u8(*v),
            Intermediate::U16(v) => visitor.visit_u16(*v),
            Intermediate::U32(v) => visitor.visit_u32(*v),
            Intermediate::U64(v) => visitor.visit_u64(*v),
            Intermediate::U128(v) => visitor.visit_u128(*v),
            Intermediate::F32(v) => visitor.visit_f32(*v),
            Intermediate::F64(v) => visitor.visit_f64(*v),
            Intermediate::Char(v) => visitor.visit_char(*v),
            Intermediate::String(v) => visitor.visit_borrowed_str(v),
            Intermediate::Bytes(v) => visitor.visit_bytes(v),
            Intermediate::Option(v) => match v {
                Some(v) => visitor.visit_some(Self::from_intermediate(v)),
                None => visitor.visit_none(),
            },
            Intermediate::UnitStruct => visitor.visit_unit(),
            Intermediate::UnitVariant(n, i) => visitor.visit_enum(EnumDeserializer::Unit(n, *i)),
            Intermediate::NewTypeStruct(v) => {
                visitor.visit_newtype_struct(Self::from_intermediate(v))
            }
            Intermediate::NewTypeVariant(n, i, v) => {
                visitor.visit_enum(EnumDeserializer::NewType(n, *i, v))
            }
            Intermediate::Seq(v) | Intermediate::Tuple(v) | Intermediate::TupleStruct(v) => visitor
                .visit_seq(SeqDeserializer {
                    values: v.as_slice(),
                    index: 0,
                }),
            Intermediate::TupleVariant(n, i, v) => {
                visitor.visit_enum(EnumDeserializer::Tuple(n, *i, v))
            }
            Intermediate::Map(v) => visitor.visit_map(MapDeserializer {
                values: v.as_slice(),
                index: 0,
            }),
            Intermediate::Struct(v) => visitor.visit_map(StructDeserializer {
                values: v.as_slice(),
                index: 0,
            }),
            Intermediate::StructVariant(n, i, v) => {
                visitor.visit_enum(EnumDeserializer::Struct(n, *i, v))
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[derive(Debug)]
pub struct SeqDeserializer<'de> {
    values: &'de [Intermediate],
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
            return seed
                .deserialize(Deserializer::from_intermediate(value))
                .map(Some);
        }
        Ok(None)
    }
}

#[derive(Debug)]
pub struct MapDeserializer<'de> {
    values: &'de [(Intermediate, Intermediate)],
    index: usize,
}

impl<'de> MapAccess<'de> for MapDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, _)) = self.values.get(self.index) {
            return seed
                .deserialize(Deserializer::from_intermediate(key))
                .map(Some);
        }
        Ok(None)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some((_, value)) = self.values.get(self.index) {
            self.index += 1;
            return seed.deserialize(Deserializer::from_intermediate(value));
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
            let key = kseed.deserialize(Deserializer::from_intermediate(key))?;
            let value = vseed.deserialize(Deserializer::from_intermediate(value))?;
            return Ok(Some((key, value)));
        }
        Ok(None)
    }
}

#[derive(Debug)]
pub struct StructDeserializer<'de> {
    values: &'de [(String, Intermediate)],
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
            return seed.deserialize(Deserializer::from_intermediate(value));
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
            let value = vseed.deserialize(Deserializer::from_intermediate(value))?;
            return Ok(Some((key, value)));
        }
        Ok(None)
    }
}

#[derive(Debug)]
enum EnumDeserializer<'de> {
    Unit(&'de str, u32),
    NewType(&'de str, u32, &'de Intermediate),
    Tuple(&'de str, u32, &'de [Intermediate]),
    Struct(&'de str, u32, &'de [(String, Intermediate)]),
}

impl<'de> EnumDeserializer<'de> {
    fn name(&self) -> &'de str {
        match self {
            Self::Unit(n, _)
            | Self::NewType(n, _, _)
            | Self::Tuple(n, _, _)
            | Self::Struct(n, _, _) => n,
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
        if let EnumDeserializer::Unit(_, _) = self {
            return Ok(());
        }
        Err(Error::ExpectedUnitVariant)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        if let EnumDeserializer::NewType(_, _, v) = self {
            return seed.deserialize(Deserializer::from_intermediate(v));
        }
        Err(Error::ExpectedNewTypeVariant)
    }

    fn tuple_variant<V>(self, _: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let EnumDeserializer::Tuple(_, _, v) = self {
            return visitor.visit_seq(SeqDeserializer {
                values: v,
                index: 0,
            });
        }
        Err(Error::ExpectedNewTypeVariant)
    }

    fn struct_variant<V>(self, _: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let EnumDeserializer::Struct(_, _, v) = self {
            return visitor.visit_map(StructDeserializer {
                values: v,
                index: 0,
            });
        }
        Err(Error::ExpectedStructVariant)
    }
}

macro_rules! impl_visit {
    ($name:ident, $type:ty) => {
        fn $name<E>(self, value: $type) -> std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value.into())
        }
    };
}

pub struct IntermediateVisitor;

impl<'de> Visitor<'de> for IntermediateVisitor {
    type Value = Intermediate;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("intermediate data representation")
    }

    impl_visit!(visit_bool, bool);
    impl_visit!(visit_i8, i8);
    impl_visit!(visit_i16, i16);
    impl_visit!(visit_i32, i32);
    impl_visit!(visit_i64, i64);
    impl_visit!(visit_i128, i128);
    impl_visit!(visit_u8, u8);
    impl_visit!(visit_u16, u16);
    impl_visit!(visit_u32, u32);
    impl_visit!(visit_u64, u64);
    impl_visit!(visit_u128, u128);
    impl_visit!(visit_f32, f32);
    impl_visit!(visit_f64, f64);
    impl_visit!(visit_char, char);

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Intermediate::String(value.to_owned()))
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Intermediate::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Intermediate::String(value))
    }

    fn visit_bytes<E>(self, value: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Intermediate::Bytes(value.to_owned()))
    }

    fn visit_borrowed_bytes<E>(self, value: &'de [u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Intermediate::Bytes(value.to_owned()))
    }

    fn visit_byte_buf<E>(self, value: Vec<u8>) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Intermediate::Bytes(value))
    }

    fn visit_none<E>(self) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Intermediate::Option(None))
    }

    fn visit_some<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Intermediate::Option(Some(Box::new(
            deserializer.deserialize_any(IntermediateVisitor)?,
        ))))
    }

    fn visit_unit<E>(self) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Intermediate::Unit)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Intermediate::NewTypeStruct(Box::new(
            deserializer.deserialize_any(IntermediateVisitor)?,
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
        Ok(Intermediate::Seq(result))
    }

    fn visit_map<A>(self, mut access: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut result = Vec::with_capacity(access.size_hint().unwrap_or_default());
        while let Some((k, v)) = access.next_entry()? {
            result.push((k, v));
        }
        Ok(Intermediate::Map(result))
    }

    // TODO: what do we do with this? this obviously can be called, but at least JSON tests don't
    // do it, have to ask smart ppl what should i do to make it work, since neither serde docs nor book shows how this works for self-describing types.
    // fn visit_enum<A>(self, mut access: A) -> std::result::Result<Self::Value, A::Error>
    // where
    //     A: EnumAccess<'de>,
    // {
    // }
}
