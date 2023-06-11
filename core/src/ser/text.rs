use crate::error::*;
use serde::Serialize;
use std::io::Write;

pub fn to_vec<T>(value: &T, config: TextConfig) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    let mut result = Vec::with_capacity(256);
    value.serialize(&mut TextSerializer::new(&mut result, config))?;
    Ok(result)
}

pub fn to_vec_compact<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    to_vec(value, TextConfig::default())
}

pub fn to_vec_pretty<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    to_vec(
        value,
        TextConfig::default().with_style(TextConfigStyle::default_pretty()),
    )
}

pub fn to_string<T>(value: &T, config: TextConfig) -> Result<String>
where
    T: Serialize + ?Sized,
{
    Ok(unsafe { String::from_utf8_unchecked(to_vec(value, config)?) })
}

pub fn to_string_compact<T>(value: &T) -> Result<String>
where
    T: Serialize + ?Sized,
{
    to_string(value, TextConfig::default())
}

pub fn to_string_pretty<T>(value: &T) -> Result<String>
where
    T: Serialize + ?Sized,
{
    to_string(
        value,
        TextConfig::default().with_style(TextConfigStyle::default_pretty()),
    )
}

#[derive(Debug, Clone)]
pub struct TextConfig {
    pub style: TextConfigStyle,
    pub numbers_with_type: bool,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            style: TextConfigStyle::Default,
            numbers_with_type: true,
        }
    }
}

impl TextConfig {
    pub fn with_style(mut self, style: TextConfigStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_numbers_with_type(mut self, mode: bool) -> Self {
        self.numbers_with_type = mode;
        self
    }
}

#[derive(Debug, Default, Clone)]
pub enum TextConfigStyle {
    #[default]
    Default,
    Pretty {
        level: usize,
        indent: Option<usize>,
    },
}

impl TextConfigStyle {
    pub fn pretty(indent: Option<usize>) -> Self {
        Self::Pretty { indent, level: 0 }
    }

    pub fn default_pretty() -> Self {
        Self::Pretty {
            indent: Some(2),
            level: 0,
        }
    }

    pub fn is_pretty(&self) -> bool {
        matches!(self, Self::Pretty { .. })
    }
}

#[derive(Debug, Default, Clone)]
pub struct TextSerializer<W>
where
    W: Write,
{
    stream: W,
    config: TextConfig,
}

impl<W> TextSerializer<W>
where
    W: Write,
{
    pub fn new(stream: W, config: TextConfig) -> Self {
        Self { stream, config }
    }

    pub fn into_inner(self) -> W {
        self.stream
    }

    fn push_level(&mut self) {
        if let TextConfigStyle::Pretty { level, .. } = &mut self.config.style {
            *level += 1;
        }
    }

    fn pop_level(&mut self) {
        if let TextConfigStyle::Pretty { level, .. } = &mut self.config.style {
            if *level > 0 {
                *level -= 1;
            }
        }
    }

    fn map_result<T>(result: std::io::Result<T>) -> Result<T> {
        result.map_err(|e| Error::Message(format!("{}", e)))
    }

    fn write_whitespace(&mut self) -> Result<()> {
        if self.config.style.is_pretty() {
            Self::map_result(write!(&mut self.stream, " "))
        } else {
            Ok(())
        }
    }

    fn write_separator(&mut self) -> Result<()> {
        if let TextConfigStyle::Pretty { indent, .. } = &self.config.style {
            if indent.is_none() {
                return self.write_raw(", ");
            }
        }
        self.write_raw(",")
    }

    fn write_new_line_indent(&mut self) -> Result<()> {
        #[allow(clippy::collapsible_match)]
        if let TextConfigStyle::Pretty { level, indent, .. } = &mut self.config.style {
            if let Some(indent) = *indent {
                Self::map_result(write!(
                    &mut self.stream,
                    "\n{:indent$}",
                    "",
                    indent = (*level) * indent
                ))?;
            }
        }
        Ok(())
    }

    fn write_raw(&mut self, value: &str) -> Result<()> {
        Self::map_result(write!(&mut self.stream, "{}", value))
    }

    fn write_from_string(&mut self, value: impl ToString, typename: &str) -> Result<()> {
        Self::map_result(write!(
            &mut self.stream,
            "{}_{}",
            value.to_string(),
            typename
        ))
    }

    fn write_str(&mut self, value: &str) -> Result<()> {
        Self::map_result(write!(&mut self.stream, "{:?}", value))
    }

    fn write_bytes(&mut self, value: &[u8]) -> Result<()> {
        Self::map_result(write!(&mut self.stream, "0x"))?;
        for byte in value {
            Self::map_result(write!(&mut self.stream, "{:02x}", byte))?;
        }
        Ok(())
    }
}

macro_rules! impl_serialize_number {
    ($name:ident, $type:ident) => {
        fn $name(self, v: $type) -> Result<Self::Ok> {
            self.write_from_string(v, stringify!($type))
        }
    };
}

impl<'a, W> serde::ser::Serializer for &'a mut TextSerializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SeqSerializer<'a, W>;
    type SerializeTuple = TupleSerializer<'a, W>;
    type SerializeTupleStruct = TupleStructSerializer<'a, W>;
    type SerializeTupleVariant = TupleVariantSerializer<'a, W>;
    type SerializeMap = MapSerializer<'a, W>;
    type SerializeStruct = StructSerializer<'a, W>;
    type SerializeStructVariant = StructVariantSerializer<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        if v {
            self.write_raw("true")
        } else {
            self.write_raw("false")
        }
    }

    impl_serialize_number!(serialize_i8, i8);
    impl_serialize_number!(serialize_i16, i16);
    impl_serialize_number!(serialize_i32, i32);
    impl_serialize_number!(serialize_i64, i64);
    impl_serialize_number!(serialize_i128, i128);
    impl_serialize_number!(serialize_u8, u8);
    impl_serialize_number!(serialize_u16, u16);
    impl_serialize_number!(serialize_u32, u32);
    impl_serialize_number!(serialize_u64, u64);
    impl_serialize_number!(serialize_u128, u128);
    impl_serialize_number!(serialize_f32, f32);
    impl_serialize_number!(serialize_f64, f64);

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        TextSerializer::<W>::map_result(write!(&mut self.stream, "'{}'", v))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.write_str(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.write_bytes(v)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.write_raw("?")
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.write_raw("?")?;
        self.write_whitespace()?;
        self.write_raw("=")?;
        self.write_whitespace()?;
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.write_raw("!")
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok> {
        self.write_raw("#!")
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.write_raw("@")?;
        self.write_raw(variant)
    }

    fn serialize_newtype_struct<T>(self, _: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.write_raw("$")?;
        self.write_whitespace()?;
        self.write_raw("=")?;
        self.write_whitespace()?;
        value.serialize(self)
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
        self.write_raw("@")?;
        self.write_raw(variant)?;
        self.write_whitespace()?;
        self.write_raw("$")?;
        self.write_whitespace()?;
        self.write_raw("=")?;
        self.write_whitespace()?;
        value.serialize(self)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq> {
        self.write_raw("[")?;
        self.push_level();
        Ok(SeqSerializer {
            stream: self,
            first: true,
        })
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple> {
        self.write_raw("(")?;
        self.push_level();
        Ok(TupleSerializer {
            stream: self,
            first: true,
        })
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.write_raw("#")?;
        self.write_whitespace()?;
        self.write_raw("(")?;
        self.push_level();
        Ok(TupleStructSerializer {
            stream: self,
            first: true,
        })
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.write_raw("@")?;
        self.write_raw(variant)?;
        self.write_whitespace()?;
        self.write_raw("(")?;
        self.push_level();
        Ok(TupleVariantSerializer {
            stream: self,
            first: true,
        })
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
        self.write_raw("{")?;
        self.push_level();
        Ok(MapSerializer {
            stream: self,
            first: true,
        })
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct> {
        self.write_raw("#")?;
        self.write_whitespace()?;
        self.write_raw("{")?;
        self.push_level();
        Ok(StructSerializer {
            stream: self,
            first: true,
        })
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.write_raw("@")?;
        self.write_raw(variant)?;
        self.write_whitespace()?;
        self.write_raw("#")?;
        self.write_whitespace()?;
        self.write_raw("{")?;
        self.push_level();
        Ok(StructVariantSerializer {
            stream: self,
            first: true,
        })
    }
}

pub struct SeqSerializer<'a, W>
where
    W: Write,
{
    stream: &'a mut TextSerializer<W>,
    first: bool,
}

impl<'a, W> serde::ser::SerializeSeq for SeqSerializer<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.first {
            self.first = false;
        } else {
            self.stream.write_separator()?;
        }
        self.stream.write_new_line_indent()?;
        value.serialize(&mut *self.stream)
    }

    fn end(self) -> Result<Self::Ok> {
        self.stream.pop_level();
        self.stream.write_new_line_indent()?;
        self.stream.write_raw("]")
    }
}

pub struct TupleSerializer<'a, W>
where
    W: Write,
{
    stream: &'a mut TextSerializer<W>,
    first: bool,
}

impl<'a, W> serde::ser::SerializeTuple for TupleSerializer<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.first {
            self.first = false;
        } else {
            self.stream.write_separator()?;
        }
        self.stream.write_new_line_indent()?;
        value.serialize(&mut *self.stream)
    }

    fn end(self) -> Result<Self::Ok> {
        self.stream.pop_level();
        self.stream.write_new_line_indent()?;
        self.stream.write_raw(")")
    }
}

pub struct TupleStructSerializer<'a, W>
where
    W: Write,
{
    stream: &'a mut TextSerializer<W>,
    first: bool,
}

impl<'a, W> serde::ser::SerializeTupleStruct for TupleStructSerializer<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.first {
            self.first = false;
        } else {
            self.stream.write_separator()?;
        }
        self.stream.write_new_line_indent()?;
        value.serialize(&mut *self.stream)
    }

    fn end(self) -> Result<Self::Ok> {
        self.stream.pop_level();
        self.stream.write_new_line_indent()?;
        self.stream.write_raw(")")
    }
}

pub struct TupleVariantSerializer<'a, W>
where
    W: Write,
{
    stream: &'a mut TextSerializer<W>,
    first: bool,
}

impl<'a, W> serde::ser::SerializeTupleVariant for TupleVariantSerializer<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.first {
            self.first = false;
        } else {
            self.stream.write_separator()?;
        }
        self.stream.write_new_line_indent()?;
        value.serialize(&mut *self.stream)
    }

    fn end(self) -> Result<Self::Ok> {
        self.stream.pop_level();
        self.stream.write_new_line_indent()?;
        self.stream.write_raw(")")
    }
}

pub struct MapSerializer<'a, W>
where
    W: Write,
{
    stream: &'a mut TextSerializer<W>,
    first: bool,
}

impl<'a, W> serde::ser::SerializeMap for MapSerializer<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.first {
            self.first = false;
        } else {
            self.stream.write_separator()?;
        }
        self.stream.write_new_line_indent()?;
        key.serialize(&mut *self.stream)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.stream.write_raw(":")?;
        self.stream.write_whitespace()?;
        value.serialize(&mut *self.stream)
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        if self.first {
            self.first = false;
        } else {
            self.stream.write_separator()?;
        }
        self.stream.write_new_line_indent()?;
        key.serialize(&mut *self.stream)?;
        self.stream.write_raw(":")?;
        self.stream.write_whitespace()?;
        value.serialize(&mut *self.stream)
    }

    fn end(self) -> Result<Self::Ok> {
        self.stream.pop_level();
        self.stream.write_new_line_indent()?;
        self.stream.write_raw("}")
    }
}

pub struct StructSerializer<'a, W>
where
    W: Write,
{
    stream: &'a mut TextSerializer<W>,
    first: bool,
}

impl<'a, W> serde::ser::SerializeStruct for StructSerializer<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.first {
            self.first = false;
        } else {
            self.stream.write_separator()?;
        }
        self.stream.write_new_line_indent()?;
        self.stream.write_raw(key)?;
        self.stream.write_raw(":")?;
        self.stream.write_whitespace()?;
        value.serialize(&mut *self.stream)
    }

    fn end(self) -> Result<()> {
        self.stream.pop_level();
        self.stream.write_new_line_indent()?;
        self.stream.write_raw("}")
    }
}

pub struct StructVariantSerializer<'a, W>
where
    W: Write,
{
    stream: &'a mut TextSerializer<W>,
    first: bool,
}

impl<'a, W> serde::ser::SerializeStructVariant for StructVariantSerializer<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.first {
            self.first = false;
        } else {
            self.stream.write_separator()?;
        }
        self.stream.write_new_line_indent()?;
        self.stream.write_raw(key)?;
        self.stream.write_raw(":")?;
        self.stream.write_whitespace()?;
        value.serialize(&mut *self.stream)
    }

    fn end(self) -> Result<()> {
        self.stream.pop_level();
        self.stream.write_new_line_indent()?;
        self.stream.write_raw("}")
    }
}
