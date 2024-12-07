use crate::de::object::ObjectVisitor;
use serde::{
    ser::{SerializeMap, SerializeSeq, SerializeStructVariant, SerializeTupleVariant},
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Number {
    SignedInteger(i64),
    UnsignedInteger(u64),
    Float(f64),
}

impl Eq for Number {}

impl Number {
    pub fn as_signed_integer(&self) -> Option<i64> {
        match self {
            Self::SignedInteger(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_unsigned_integer(&self) -> Option<u64> {
        match self {
            Self::UnsignedInteger(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }
}

impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::SignedInteger(v) => serializer.serialize_i64(*v),
            Self::UnsignedInteger(v) => serializer.serialize_u64(*v),
            Self::Float(v) => serializer.serialize_f64(*v),
        }
    }
}

macro_rules! impl_number_from {
    ($type:ty => $variant:ident) => {
        impl From<$type> for Number {
            fn from(value: $type) -> Self {
                Self::$variant(value as _)
            }
        }
    };
}

impl_number_from!(i8 => SignedInteger);
impl_number_from!(i16 => SignedInteger);
impl_number_from!(i32 => SignedInteger);
impl_number_from!(i64 => SignedInteger);
impl_number_from!(u8 => UnsignedInteger);
impl_number_from!(u16 => UnsignedInteger);
impl_number_from!(u32 => UnsignedInteger);
impl_number_from!(u64 => UnsignedInteger);
impl_number_from!(f32 => Float);
impl_number_from!(f64 => Float);

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd)]
pub enum Variant {
    #[default]
    Unit,
    Wrapper(Box<Object>),
    Array(Vec<Object>),
    Map(Vec<(String, Object)>),
}

impl Variant {
    pub fn unit() -> Self {
        Self::Unit
    }

    pub fn wrapper(value: impl Into<Object>) -> Self {
        Self::Wrapper(Box::new(value.into()))
    }

    pub fn array() -> Self {
        Self::Array(Default::default())
    }

    pub fn array_from<T: Into<Object>>(value: impl IntoIterator<Item = T>) -> Self {
        Self::Array(value.into_iter().map(|item| item.into()).collect())
    }

    pub fn item(self, value: impl Into<Object>) -> Self {
        match self {
            Self::Array(mut result) => {
                result.push(value.into());
                Self::Array(result)
            }
            _ => self,
        }
    }

    pub fn map() -> Self {
        Self::Map(Default::default())
    }

    pub fn map_from<K: ToString, V: Into<Object>>(value: impl IntoIterator<Item = (K, V)>) -> Self {
        Self::Map(
            value
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.into()))
                .collect(),
        )
    }

    pub fn property(self, key: impl ToString, value: impl Into<Object>) -> Self {
        match self {
            Self::Map(mut result) => {
                let key = key.to_string();
                let value = value.into();
                if let Some((_, item)) = result.iter_mut().find(|(k, _)| k == &key) {
                    *item = value;
                } else {
                    result.push((key, value));
                }
                Self::Map(result)
            }
            _ => self,
        }
    }

    pub fn as_unit(&self) -> Option<()> {
        match self {
            Self::Unit => Some(()),
            _ => None,
        }
    }

    pub fn as_wrapper(&self) -> Option<&Object> {
        match self {
            Self::Wrapper(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Object]> {
        match self {
            Self::Array(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&[(String, Object)]> {
        match self {
            Self::Map(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd)]
pub enum Object {
    #[default]
    Unit,
    Bool(bool),
    Number(Number),
    String(String),
    Wrapper(Box<Object>),
    Array(Vec<Object>),
    Map(Vec<(Object, Object)>),
    Option(Option<Box<Object>>),
    Variant {
        name: String,
        value: Box<Variant>,
    },
}

impl Object {
    pub fn unit() -> Self {
        Self::Unit
    }

    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    pub fn number(value: impl Into<Number>) -> Self {
        Self::Number(value.into())
    }

    pub fn string(value: impl ToString) -> Self {
        Self::String(value.to_string())
    }

    pub fn wrapper(value: impl Into<Object>) -> Self {
        Self::Wrapper(Box::new(value.into()))
    }

    pub fn array() -> Self {
        Self::Array(Default::default())
    }

    pub fn array_from<T: Into<Object>>(value: impl IntoIterator<Item = T>) -> Self {
        Self::Array(value.into_iter().map(|item| item.into()).collect())
    }

    pub fn item(self, value: impl Into<Object>) -> Self {
        match self {
            Self::Array(mut result) => {
                result.push(value.into());
                Self::Array(result)
            }
            _ => self,
        }
    }

    pub fn map() -> Self {
        Self::Map(Default::default())
    }

    pub fn map_from<K: Into<Object>, V: Into<Object>>(
        value: impl IntoIterator<Item = (K, V)>,
    ) -> Self {
        Self::Map(
            value
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        )
    }

    pub fn property(self, key: impl Into<Object>, value: impl Into<Object>) -> Self {
        match self {
            Self::Map(mut result) => {
                let key = key.into();
                let value = value.into();
                if let Some((_, item)) = result.iter_mut().find(|(k, _)| k == &key) {
                    *item = value;
                } else {
                    result.push((key, value));
                }
                Self::Map(result)
            }
            _ => self,
        }
    }

    pub fn option(value: Option<impl Into<Object>>) -> Self {
        Self::Option(value.map(|value| Box::new(value.into())))
    }

    pub fn variant(name: impl ToString, value: Variant) -> Self {
        Self::Variant {
            name: name.to_string(),
            value: Box::new(value),
        }
    }

    pub fn as_unit(&self) -> Option<()> {
        match self {
            Self::Unit => Some(()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<&Number> {
        match self {
            Self::Number(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v.as_str()),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::String(v) => Some(v.to_owned()),
            _ => None,
        }
    }

    pub fn as_wrapper(&self) -> Option<&Object> {
        match self {
            Self::Wrapper(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Object]> {
        match self {
            Self::Array(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&[(Object, Object)]> {
        match self {
            Self::Map(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_option(&self) -> Option<&Object> {
        match self {
            Self::Option(v) => v.as_ref().map(|v| &**v),
            _ => None,
        }
    }

    pub fn as_variant(&self) -> Option<(&str, &Variant)> {
        match self {
            Self::Variant { name, value } => Some((name.as_str(), value)),
            _ => None,
        }
    }
}

impl Serialize for Object {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Unit => serializer.serialize_unit(),
            Self::Bool(v) => serializer.serialize_bool(*v),
            Self::Number(v) => v.serialize(serializer),
            Self::String(v) => serializer.serialize_str(v),
            Self::Wrapper(v) => v.serialize(serializer),
            Self::Array(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Self::Map(v) => {
                let mut map = serializer.serialize_map(Some(v.len()))?;
                for (k, v) in v {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Self::Option(v) => match v {
                Some(v) => serializer.serialize_some(v),
                None => serializer.serialize_none(),
            },
            Self::Variant { name, value } => match &**value {
                Variant::Unit => serializer.serialize_unit_variant("Object", 0, unsafe {
                    std::mem::transmute::<&str, &str>(name.as_str())
                }),
                Variant::Wrapper(v) => serializer.serialize_newtype_variant(
                    "Object",
                    0,
                    unsafe { std::mem::transmute::<&str, &str>(name.as_str()) },
                    v,
                ),
                Variant::Array(v) => {
                    let mut tv = serializer.serialize_tuple_variant(
                        "Object",
                        0,
                        unsafe { std::mem::transmute::<&str, &str>(name.as_str()) },
                        v.len(),
                    )?;
                    for item in v {
                        tv.serialize_field(item)?;
                    }
                    tv.end()
                }
                Variant::Map(v) => {
                    let mut sv = serializer.serialize_struct_variant(
                        "Object",
                        0,
                        unsafe { std::mem::transmute::<&str, &str>(name.as_str()) },
                        v.len(),
                    )?;
                    for (k, v) in v {
                        sv.serialize_field(
                            unsafe { std::mem::transmute::<&str, &str>(k.as_str()) },
                            v,
                        )?;
                    }
                    sv.end()
                }
            },
        }
    }
}

impl<'de> Deserialize<'de> for Object {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ObjectVisitor)
    }
}
