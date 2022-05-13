use crate::value::intermediate::Intermediate;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Message(String),
    ExpectedMapEntry,
    ExpectedStructField,
    ExpectedUnitVariant,
    ExpectedNewTypeVariant,
    ExpectedTupleVariant,
    ExpectedStructVariant,
    NotPartial(Intermediate),
    NotSeq(Intermediate),
    NotTuple(Intermediate),
    NotMap(Intermediate),
    NotStruct(Intermediate),
    CannotAdd(Intermediate),
    CannotRemove(Intermediate),
    /// (value, old size, new size)
    InvalidSize(Intermediate, usize, usize),
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::ExpectedMapEntry => formatter.write_str("expected map entry"),
            Error::ExpectedStructField => formatter.write_str("expected struct field"),
            Error::ExpectedUnitVariant => formatter.write_str("expected unit variant"),
            Error::ExpectedNewTypeVariant => formatter.write_str("expected newtype variant"),
            Error::ExpectedTupleVariant => formatter.write_str("expected tuple variant"),
            Error::ExpectedStructVariant => formatter.write_str("expected struct variant"),
            Error::NotPartial(_) => formatter.write_str("value is not a partial"),
            Error::NotSeq(_) => formatter.write_str("value is not a sequence"),
            Error::NotTuple(_) => formatter.write_str("value is not a tuple"),
            Error::NotMap(_) => formatter.write_str("value is not a map"),
            Error::NotStruct(_) => formatter.write_str("value is not a struct"),
            Error::CannotAdd(_) => formatter.write_str("cannot add value here"),
            Error::CannotRemove(_) => formatter.write_str("cannot remove value here"),
            Error::InvalidSize(_, _, _) => formatter.write_str("invalid size here"),
        }
    }
}

impl std::error::Error for Error {}
