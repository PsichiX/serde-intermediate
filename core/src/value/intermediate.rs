use crate::{
    de::intermediate::IntermediateVisitor, reflect::ReflectIntermediate, versioning::Change,
};
use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::collections::{HashMap, HashSet};

/// Serde intermediate data representation.
///
/// # Example
/// ```rust
/// use std::time::SystemTime;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// enum Login {
///     Email(String),
///     SocialMedia{
///         service: String,
///         token: String,
///         last_login: Option<SystemTime>,
///     }
/// }
///
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// struct Person {
///     // (first name, last name)
///     name: (String, String),
///     age: usize,
///     login: Login,
/// }
///
/// let data = Person {
///     name: ("John".to_owned(), "Smith".to_owned()),
///     age: 40,
///     login: Login::Email("john.smith@gmail.com".to_owned()),
/// };
/// let serialized = serde_intermediate::to_intermediate(&data).unwrap();
/// let deserialized = serde_intermediate::from_intermediate(&serialized).unwrap();
/// assert_eq!(data, deserialized);
/// ```
#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub enum Intermediate {
    /// Unit value: `()`.
    #[default]
    Unit,
    /// Bool value: `true`.
    Bool(bool),
    /// 8-bit signed integer value: `42`.
    I8(i8),
    /// 16-bit signed integer value: `42`.
    I16(i16),
    /// 32-bit signed integer value: `42`.
    I32(i32),
    /// 64-bit signed integer value: `42`.
    I64(i64),
    /// 128-bit signed integer value: `42`.
    I128(i128),
    /// 8-bit unsigned integer value: `42`.
    U8(u8),
    /// 16-bit unsigned integer value: `42`.
    U16(u16),
    /// 32-bit unsigned integer value: `42`.
    U32(u32),
    /// 64-bit unsigned integer value: `42`.
    U64(u64),
    /// 128-bit unsigned integer value: `42`.
    U128(u128),
    /// 32-bit floating point value: `3.14`.
    F32(f32),
    /// 64-bit floating point value: `3.14`.
    F64(f64),
    /// Single character value: `'@'`.
    Char(char),
    /// String value: `"Hello World!"`.
    String(String),
    /// Bytes buffer.
    Bytes(Vec<u8>),
    /// Option value: `Some(42)`.
    Option(
        /// Value.
        Option<Box<Self>>,
    ),
    /// Structure: `struct Foo;`.
    UnitStruct,
    /// Enum unit variant: `enum Foo { Bar }`.
    UnitVariant(
        /// Variant name.
        String,
    ),
    /// Newtype struct: `struct Foo(bool);`.
    NewTypeStruct(Box<Self>),
    /// Enum newtype variant: `enum Foo { Bar(bool) }`.
    NewTypeVariant(
        /// Variant name.
        String,
        /// Value.
        Box<Self>,
    ),
    /// Sequence/list: `Vec<usize>`, `[usize]`.
    Seq(
        /// Items.
        Vec<Self>,
    ),
    /// Tuple: `(bool, char)`.
    Tuple(
        /// Fields.
        Vec<Self>,
    ),
    /// Tuple struct: `struct Foo(bool, char)`.
    TupleStruct(
        /// Fields.
        Vec<Self>,
    ),
    /// Tuple variant: `enum Foo { Bar(bool, char) }`.
    TupleVariant(
        /// Variant name.
        String,
        /// Fields.
        Vec<Self>,
    ),
    /// Map: `HashMap<String, usize>`.
    Map(
        /// Entries: `(key, value)`.
        Vec<(Self, Self)>,
    ),
    /// Struct: `struct Foo { a: bool, b: char }`.
    Struct(
        /// Fields: `(name, value)`.
        Vec<(String, Self)>,
    ),
    /// Enum struct variant: `enum Foo { Bar { a: bool, b: char } }`.
    StructVariant(
        /// Variant name.
        String,
        /// Fields: `(name, value)`.
        Vec<(String, Self)>,
    ),
}

impl Eq for Intermediate {}

impl Intermediate {
    pub fn unit_struct() -> Self {
        Self::UnitStruct
    }

    pub fn unit_variant<T>(name: T) -> Self
    where
        T: ToString,
    {
        Self::UnitVariant(name.to_string())
    }

    pub fn newtype_struct(value: Self) -> Self {
        Self::NewTypeStruct(Box::new(value))
    }

    pub fn newtype_variant<T>(name: T, value: Self) -> Self
    where
        T: ToString,
    {
        Self::NewTypeVariant(name.to_string(), Box::new(value))
    }

    pub fn seq() -> Self {
        Self::Seq(vec![])
    }

    pub fn tuple() -> Self {
        Self::Tuple(vec![])
    }

    pub fn tuple_struct() -> Self {
        Self::TupleStruct(vec![])
    }

    pub fn tuple_variant<T>(name: T) -> Self
    where
        T: ToString,
    {
        Self::TupleVariant(name.to_string(), vec![])
    }

    pub fn map() -> Self {
        Self::Map(vec![])
    }

    pub fn struct_type() -> Self {
        Self::Struct(vec![])
    }

    pub fn struct_variant<T>(name: T) -> Self
    where
        T: ToString,
    {
        Self::StructVariant(name.to_string(), vec![])
    }

    pub fn item<T>(mut self, value: T) -> Self
    where
        T: Into<Self>,
    {
        match &mut self {
            Self::Seq(v) | Self::Tuple(v) | Self::TupleStruct(v) | Self::TupleVariant(_, v) => {
                v.push(value.into())
            }
            _ => {}
        }
        self
    }

    pub fn property<K, T>(mut self, key: K, value: T) -> Self
    where
        K: Into<Self>,
        T: Into<Self>,
    {
        if let Self::Map(v) = &mut self {
            v.push((key.into(), value.into()));
        }
        self
    }

    pub fn field<K, T>(mut self, key: K, value: T) -> Self
    where
        K: ToString,
        T: Into<Self>,
    {
        match &mut self {
            Self::Struct(v) | Self::StructVariant(_, v) => v.push((key.to_string(), value.into())),
            _ => {}
        }
        self
    }

    pub fn total_bytesize(&self) -> usize {
        fn string_bytesize(v: &str) -> usize {
            std::mem::size_of_val(v.as_bytes())
        }

        std::mem::size_of_val(self)
            + match self {
                Self::String(v) => string_bytesize(v),
                Self::Bytes(v) => v.len() * std::mem::size_of::<u8>(),
                Self::Option(v) => v.as_ref().map(|v| v.total_bytesize()).unwrap_or_default(),
                Self::UnitVariant(n) => string_bytesize(n),
                Self::NewTypeStruct(v) => v.total_bytesize(),
                Self::NewTypeVariant(n, v) => string_bytesize(n) + v.total_bytesize(),
                Self::Seq(v) | Self::Tuple(v) | Self::TupleStruct(v) => {
                    v.iter().map(|v| v.total_bytesize()).sum()
                }
                Self::TupleVariant(n, v) => {
                    string_bytesize(n) + v.iter().map(|v| v.total_bytesize()).sum::<usize>()
                }
                Self::Map(v) => v
                    .iter()
                    .map(|(k, v)| k.total_bytesize() + v.total_bytesize())
                    .sum(),
                Self::Struct(v) => v
                    .iter()
                    .map(|(k, v)| string_bytesize(k) + v.total_bytesize())
                    .sum(),
                Self::StructVariant(n, v) => {
                    string_bytesize(n)
                        + v.iter()
                            .map(|(k, v)| string_bytesize(k) + v.total_bytesize())
                            .sum::<usize>()
                }
                _ => 0,
            }
    }
}

impl ReflectIntermediate for Intermediate {
    fn patch_change(&mut self, change: &Change) {
        if let Ok(Some(v)) = change.patch(self) {
            *self = v;
        }
    }
}

impl std::fmt::Display for Intermediate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = crate::to_string_compact(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", content)
    }
}

macro_rules! impl_as {
    (@copy $method:ident : $type:ty => $variant:ident) => {
        pub fn $method(&self) -> Option<$type> {
            match self {
                Self::$variant(v) => Some(*v),
                _ => None,
            }
        }
    };
    (@ref $method:ident : $type:ty => $variant:ident) => {
        pub fn $method(&self) -> Option<$type> {
            match self {
                Self::$variant(v) => Some(v),
                _ => None,
            }
        }
    };
}

impl Intermediate {
    pub fn as_unit(&self) -> Option<()> {
        match self {
            Self::Unit => Some(()),
            _ => None,
        }
    }

    impl_as! {@copy as_bool : bool => Bool}
    impl_as! {@copy as_i8 : i8 => I8}
    impl_as! {@copy as_i16 : i16 => I16}
    impl_as! {@copy as_i32 : i32 => I32}
    impl_as! {@copy as_i64 : i64 => I64}
    impl_as! {@copy as_i128 : i128 => I128}
    impl_as! {@copy as_u8 : u8 => U8}
    impl_as! {@copy as_u16 : u16 => U16}
    impl_as! {@copy as_u32 : u32 => U32}
    impl_as! {@copy as_u64 : u64 => U64}
    impl_as! {@copy as_u128 : u128 => U128}
    impl_as! {@copy as_f32 : f32 => F32}
    impl_as! {@copy as_f64 : f64 => F64}
    impl_as! {@copy as_char : char => Char}
    impl_as! {@ref as_str : &str => String}
    impl_as! {@ref as_bytes : &[u8] => Bytes}
    impl_as! {@ref as_seq : &[Self] => Seq}
    impl_as! {@ref as_tuple : &[Self] => Tuple}
    impl_as! {@ref as_tuple_struct : &[Self] => TupleStruct}
    impl_as! {@ref as_map : &[(Self, Self)] => Map}
    impl_as! {@ref as_struct : &[(String, Self)] => Struct}

    pub fn as_option(&self) -> Option<&Self> {
        match self {
            Self::Option(Some(v)) => Some(v),
            _ => None,
        }
    }

    pub fn as_unit_variant(&self) -> Option<&str> {
        match self {
            Self::UnitVariant(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_new_type_struct(&self) -> Option<&Self> {
        match self {
            Self::NewTypeStruct(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_new_type_variant(&self) -> Option<(&str, &Self)> {
        match self {
            Self::NewTypeVariant(n, v) => Some((n, v)),
            _ => None,
        }
    }

    pub fn as_tuple_variant(&self) -> Option<(&str, &[Self])> {
        match self {
            Self::TupleVariant(n, v) => Some((n, v)),
            _ => None,
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn as_struct_variant(&self) -> Option<(&str, &[(String, Self)])> {
        match self {
            Self::StructVariant(n, v) => Some((n, v)),
            _ => None,
        }
    }
}

macro_rules! impl_from_wrap {
    ($type:ty, $variant:ident) => {
        impl From<$type> for Intermediate {
            fn from(v: $type) -> Self {
                Self::$variant(v)
            }
        }
    };
}

impl From<()> for Intermediate {
    fn from(_: ()) -> Self {
        Self::Unit
    }
}

impl_from_wrap!(bool, Bool);
impl_from_wrap!(i8, I8);
impl_from_wrap!(i16, I16);
impl_from_wrap!(i32, I32);
impl_from_wrap!(i64, I64);
impl_from_wrap!(i128, I128);
impl_from_wrap!(u8, U8);
impl_from_wrap!(u16, U16);
impl_from_wrap!(u32, U32);
impl_from_wrap!(u64, U64);
impl_from_wrap!(u128, U128);
impl_from_wrap!(f32, F32);
impl_from_wrap!(f64, F64);
impl_from_wrap!(char, Char);
impl_from_wrap!(String, String);
impl_from_wrap!(Vec<u8>, Bytes);

impl From<isize> for Intermediate {
    fn from(v: isize) -> Self {
        Self::I64(v as _)
    }
}

impl From<usize> for Intermediate {
    fn from(v: usize) -> Self {
        Self::U64(v as _)
    }
}

impl From<&str> for Intermediate {
    fn from(v: &str) -> Self {
        Self::String(v.to_owned())
    }
}

impl From<Option<Intermediate>> for Intermediate {
    fn from(v: Option<Self>) -> Self {
        Self::Option(v.map(Box::new))
    }
}

impl From<Result<Intermediate, Intermediate>> for Intermediate {
    fn from(v: Result<Self, Self>) -> Self {
        match v {
            Ok(v) => Self::NewTypeVariant("Ok".to_owned(), Box::new(v)),
            Err(v) => Self::NewTypeVariant("Err".to_owned(), Box::new(v)),
        }
    }
}

impl<const N: usize> From<[Intermediate; N]> for Intermediate {
    fn from(v: [Self; N]) -> Self {
        Self::Seq(v.to_vec())
    }
}

impl From<(Intermediate,)> for Intermediate {
    fn from(v: (Self,)) -> Self {
        Self::Tuple(vec![v.0])
    }
}

macro_rules! impl_from_tuple {
    ( $( $id:ident ),+ ) => {
        impl< $( $id ),+ > From<( $( $id ),+ )> for Intermediate where $( $id: Into<Intermediate> ),+ {
            #[allow(non_snake_case)]
            fn from(v: ( $( $id ),+ )) -> Self {
                let ( $( $id ),+ ) = v;
                Self::Tuple(vec![$( $id.into() ),+])
            }
        }
    };
}

impl_from_tuple!(A, B);
impl_from_tuple!(A, B, C);
impl_from_tuple!(A, B, C, D);
impl_from_tuple!(A, B, C, D, E);
impl_from_tuple!(A, B, C, D, E, F);
impl_from_tuple!(A, B, C, D, E, F, G);
impl_from_tuple!(A, B, C, D, E, F, G, H);
impl_from_tuple!(A, B, C, D, E, F, G, H, I);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, X);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, X, Y);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, X, Y, Z);

impl From<Vec<Intermediate>> for Intermediate {
    fn from(v: Vec<Self>) -> Self {
        Self::Seq(v)
    }
}

impl From<HashSet<Intermediate>> for Intermediate {
    fn from(v: HashSet<Self>) -> Self {
        Self::Seq(v.into_iter().collect())
    }
}

impl From<HashMap<Intermediate, Intermediate>> for Intermediate {
    fn from(v: HashMap<Self, Self>) -> Self {
        Self::Map(v.into_iter().collect())
    }
}

impl From<HashMap<String, Intermediate>> for Intermediate {
    fn from(v: HashMap<String, Self>) -> Self {
        Self::Struct(v.into_iter().collect())
    }
}

impl Serialize for Intermediate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Unit => serializer.serialize_unit(),
            Self::Bool(v) => serializer.serialize_bool(*v),
            Self::I8(v) => serializer.serialize_i8(*v),
            Self::I16(v) => serializer.serialize_i16(*v),
            Self::I32(v) => serializer.serialize_i32(*v),
            Self::I64(v) => serializer.serialize_i64(*v),
            Self::I128(v) => serializer.serialize_i128(*v),
            Self::U8(v) => serializer.serialize_u8(*v),
            Self::U16(v) => serializer.serialize_u16(*v),
            Self::U32(v) => serializer.serialize_u32(*v),
            Self::U64(v) => serializer.serialize_u64(*v),
            Self::U128(v) => serializer.serialize_u128(*v),
            Self::F32(v) => serializer.serialize_f32(*v),
            Self::F64(v) => serializer.serialize_f64(*v),
            Self::Char(v) => serializer.serialize_char(*v),
            Self::String(v) => serializer.serialize_str(v),
            Self::Bytes(v) => serializer.serialize_bytes(v),
            Self::Option(v) => match v {
                Some(v) => serializer.serialize_some(v),
                None => serializer.serialize_none(),
            },
            Self::UnitStruct => serializer.serialize_unit_struct("Intermediate"),
            Self::UnitVariant(n) => serializer.serialize_unit_variant("Intermediate", 0, unsafe {
                std::mem::transmute::<&str, &str>(n.as_str())
            }),
            Self::NewTypeStruct(v) => serializer.serialize_newtype_struct("Intermediate", v),
            Self::NewTypeVariant(n, v) => serializer.serialize_newtype_variant(
                "Intermediate",
                0,
                unsafe { std::mem::transmute::<&str, &str>(n.as_str()) },
                v,
            ),
            Self::Seq(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Self::Tuple(v) => {
                let mut tup = serializer.serialize_tuple(v.len())?;
                for item in v {
                    tup.serialize_element(item)?;
                }
                tup.end()
            }
            Self::TupleStruct(v) => {
                let mut tup = serializer.serialize_tuple_struct("Intermediate", v.len())?;
                for item in v {
                    tup.serialize_field(item)?;
                }
                tup.end()
            }
            Self::TupleVariant(n, v) => {
                let mut tv = serializer.serialize_tuple_variant(
                    "Intermediate",
                    0,
                    unsafe { std::mem::transmute::<&str, &str>(n.as_str()) },
                    v.len(),
                )?;
                for item in v {
                    tv.serialize_field(item)?;
                }
                tv.end()
            }
            Self::Map(v) => {
                let mut map = serializer.serialize_map(Some(v.len()))?;
                for (k, v) in v {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Self::Struct(v) => {
                let mut st = serializer.serialize_struct("Intermediate", v.len())?;
                for (k, v) in v {
                    st.serialize_field(
                        unsafe { std::mem::transmute::<&str, &str>(k.as_str()) },
                        v,
                    )?;
                }
                st.end()
            }
            Self::StructVariant(n, v) => {
                let mut sv = serializer.serialize_struct_variant(
                    "Intermediate",
                    0,
                    unsafe { std::mem::transmute::<&str, &str>(n.as_str()) },
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
        }
    }
}

impl<'de> Deserialize<'de> for Intermediate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(IntermediateVisitor)
    }
}
