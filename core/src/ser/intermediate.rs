use crate::{error::*, value::intermediate::*};
use serde::Serialize;

macro_rules! impl_serialize {
    ($name:ident, $variant:ident, $type:ident) => {
        fn $name(self, v: $type) -> Result<Self::Ok> {
            Ok(Intermediate::$variant(v))
        }
    };
}

pub fn serialize<T>(value: &T) -> Result<Intermediate>
where
    T: Serialize,
{
    value.serialize(Serializer)
}

pub struct Serializer;

impl serde::ser::Serializer for Serializer {
    type Ok = Intermediate;
    type Error = Error;
    type SerializeSeq = SeqSerializer;
    type SerializeTuple = TupleSerializer;
    type SerializeTupleStruct = TupleStructSerializer;
    type SerializeTupleVariant = TupleVariantSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = StructSerializer;
    type SerializeStructVariant = StructVariantSerializer;

    impl_serialize!(serialize_bool, Bool, bool);
    impl_serialize!(serialize_i8, I8, i8);
    impl_serialize!(serialize_i16, I16, i16);
    impl_serialize!(serialize_i32, I32, i32);
    impl_serialize!(serialize_i64, I64, i64);
    impl_serialize!(serialize_i128, I128, i128);
    impl_serialize!(serialize_u8, U8, u8);
    impl_serialize!(serialize_u16, U16, u16);
    impl_serialize!(serialize_u32, U32, u32);
    impl_serialize!(serialize_u64, U64, u64);
    impl_serialize!(serialize_u128, U128, u128);
    impl_serialize!(serialize_f32, F32, f32);
    impl_serialize!(serialize_f64, F64, f64);
    impl_serialize!(serialize_char, Char, char);

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(Intermediate::String(v.to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        Ok(Intermediate::Bytes(v.to_owned()))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(Intermediate::Option(None))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Ok(Intermediate::Option(Some(Box::new(value.serialize(self)?))))
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(Intermediate::Unit)
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok> {
        Ok(Intermediate::UnitStruct)
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(Intermediate::UnitVariant(variant.to_owned()))
    }

    fn serialize_newtype_struct<T>(self, _: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Ok(Intermediate::NewTypeStruct(Box::new(
            value.serialize(self)?,
        )))
    }

    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Ok(Intermediate::NewTypeVariant(
            variant.to_owned(),
            Box::new(value.serialize(self)?),
        ))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SeqSerializer {
            values: match len {
                Some(len) => Vec::with_capacity(len),
                None => vec![],
            },
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        Ok(TupleSerializer {
            values: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(TupleStructSerializer {
            values: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(TupleVariantSerializer {
            variant: variant.to_owned(),
            values: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapSerializer {
            values: match len {
                Some(len) => Vec::with_capacity(len),
                None => vec![],
            },
        })
    }

    fn serialize_struct(self, _: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(StructSerializer {
            values: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(StructVariantSerializer {
            variant: variant.to_owned(),
            values: Vec::with_capacity(len),
        })
    }
}

pub struct SeqSerializer {
    values: Vec<Intermediate>,
}

impl serde::ser::SerializeSeq for SeqSerializer {
    type Ok = Intermediate;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Intermediate::Seq(self.values))
    }
}

pub struct TupleSerializer {
    values: Vec<Intermediate>,
}

impl serde::ser::SerializeTuple for TupleSerializer {
    type Ok = Intermediate;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Intermediate::Tuple(self.values))
    }
}

pub struct TupleStructSerializer {
    values: Vec<Intermediate>,
}

impl serde::ser::SerializeTupleStruct for TupleStructSerializer {
    type Ok = Intermediate;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Intermediate::TupleStruct(self.values))
    }
}

pub struct TupleVariantSerializer {
    variant: String,
    values: Vec<Intermediate>,
}

impl serde::ser::SerializeTupleVariant for TupleVariantSerializer {
    type Ok = Intermediate;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Intermediate::TupleVariant(self.variant, self.values))
    }
}

pub struct MapSerializer {
    values: Vec<(Intermediate, Intermediate)>,
}

impl serde::ser::SerializeMap for MapSerializer {
    type Ok = Intermediate;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values
            .push((key.serialize(Serializer)?, Intermediate::Unit));
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.last_mut().unwrap().1 = value.serialize(Serializer)?;
        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        self.values
            .push((key.serialize(Serializer)?, value.serialize(Serializer)?));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Intermediate::Map(self.values))
    }
}

pub struct StructSerializer {
    values: Vec<(String, Intermediate)>,
}

impl serde::ser::SerializeStruct for StructSerializer {
    type Ok = Intermediate;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values
            .push((key.to_owned(), value.serialize(Serializer)?));
        Ok(())
    }

    fn end(self) -> Result<Intermediate> {
        Ok(Intermediate::Struct(self.values))
    }
}

pub struct StructVariantSerializer {
    variant: String,
    values: Vec<(String, Intermediate)>,
}

impl serde::ser::SerializeStructVariant for StructVariantSerializer {
    type Ok = Intermediate;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values
            .push((key.to_owned(), value.serialize(Serializer)?));
        Ok(())
    }

    fn end(self) -> Result<Intermediate> {
        Ok(Intermediate::StructVariant(self.variant, self.values))
    }
}
