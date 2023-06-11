use crate::{error::*, value::object::*};
use serde::Serialize;

pub fn serialize<T>(value: &T) -> Result<Object>
where
    T: Serialize + ?Sized,
{
    value.serialize(Serializer)
}

pub struct Serializer;

impl serde::ser::Serializer for Serializer {
    type Ok = Object;
    type Error = Error;
    type SerializeSeq = SeqSerializer;
    type SerializeTuple = TupleSerializer;
    type SerializeTupleStruct = TupleStructSerializer;
    type SerializeTupleVariant = TupleVariantSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = StructSerializer;
    type SerializeStructVariant = StructVariantSerializer;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(Object::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        Ok(Object::Number(Number::SignedInteger(v as _)))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        Ok(Object::Number(Number::SignedInteger(v as _)))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        Ok(Object::Number(Number::SignedInteger(v as _)))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        Ok(Object::Number(Number::SignedInteger(v)))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        Ok(Object::Number(Number::UnsignedInteger(v as _)))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        Ok(Object::Number(Number::UnsignedInteger(v as _)))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        Ok(Object::Number(Number::UnsignedInteger(v as _)))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(Object::Number(Number::UnsignedInteger(v)))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        Ok(Object::Number(Number::Float(v as _)))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(Object::Number(Number::Float(v)))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        Ok(Object::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(Object::String(v.to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        Ok(Object::Array(
            v.iter()
                .map(|v| Object::Number(Number::UnsignedInteger(*v as _)))
                .collect(),
        ))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(Object::Option(None))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Ok(Object::Option(Some(Box::new(value.serialize(self)?))))
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(Object::Unit)
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok> {
        Ok(Object::Unit)
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(Object::Variant {
            name: variant.to_owned(),
            value: Box::new(Variant::Unit),
        })
    }

    fn serialize_newtype_struct<T>(self, _: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Ok(Object::Wrapper(Box::new(value.serialize(self)?)))
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
        Ok(Object::Variant {
            name: variant.to_owned(),
            value: Box::new(Variant::Wrapper(Box::new(value.serialize(self)?))),
        })
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
    values: Vec<Object>,
}

impl serde::ser::SerializeSeq for SeqSerializer {
    type Ok = Object;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Object::Array(self.values))
    }
}

pub struct TupleSerializer {
    values: Vec<Object>,
}

impl serde::ser::SerializeTuple for TupleSerializer {
    type Ok = Object;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Object::Array(self.values))
    }
}

pub struct TupleStructSerializer {
    values: Vec<Object>,
}

impl serde::ser::SerializeTupleStruct for TupleStructSerializer {
    type Ok = Object;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Object::Array(self.values))
    }
}

pub struct TupleVariantSerializer {
    variant: String,
    values: Vec<Object>,
}

impl serde::ser::SerializeTupleVariant for TupleVariantSerializer {
    type Ok = Object;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Object::Variant {
            name: self.variant,
            value: Box::new(Variant::Array(self.values)),
        })
    }
}

pub struct MapSerializer {
    values: Vec<(Object, Object)>,
}

impl serde::ser::SerializeMap for MapSerializer {
    type Ok = Object;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.push((key.serialize(Serializer)?, Object::Unit));
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
        Ok(Object::Map(self.values))
    }
}

pub struct StructSerializer {
    values: Vec<(Object, Object)>,
}

impl serde::ser::SerializeStruct for StructSerializer {
    type Ok = Object;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values
            .push((Object::String(key.to_owned()), value.serialize(Serializer)?));
        Ok(())
    }

    fn end(self) -> Result<Object> {
        Ok(Object::Map(self.values))
    }
}

pub struct StructVariantSerializer {
    variant: String,
    values: Vec<(String, Object)>,
}

impl serde::ser::SerializeStructVariant for StructVariantSerializer {
    type Ok = Object;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values
            .push((key.to_owned(), value.serialize(Serializer)?));
        Ok(())
    }

    fn end(self) -> Result<Object> {
        Ok(Object::Variant {
            name: self.variant,
            value: Box::new(Variant::Map(self.values)),
        })
    }
}
