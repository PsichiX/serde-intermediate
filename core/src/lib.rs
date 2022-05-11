#[cfg(test)]
mod tests;

pub mod de;
pub mod error;
pub mod reflect;
pub mod ser;
pub mod value;
pub mod versioning;

pub use crate::{
    de::intermediate::deserialize, de::intermediate::deserialize as from_intermediate,
    error::Error, reflect::ReflectIntermediate, ser::intermediate::serialize,
    ser::intermediate::serialize as to_intermediate, value::intermediate::Intermediate,
    versioning::*,
};

#[cfg(feature = "derive")]
pub use serde_intermediate_derive::*;
