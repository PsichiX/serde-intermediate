//! # serde-intermediate
//! Intermediate representation for Rust's Serde serialization
//!
//! ---
//!
//! ## Table of contents
//!
//! 1. [Goals](#goals)
//! 1. [Installation](#installation)
//! 1. [Examples](#examples)
//!
//! ---
//!
//! ## Goals
//!
//! This crate was made to solve these particular problems:
//!
//! - Provide untyped (obviously "fat") runtime value representation used as exchange data ready to be deserialized on demand into typed data, for data where serializable tagged trait objects don't work on taret platforms.
//!
//!     Example: data stored in interpreted language runtime value.
//!
//! - Support for more interpreted than exact data conversion (_if it quacks like a duck, treat it like a duck_) which is default behavior, optionally force to read exact layout stored in value.
//!
//!     Example: more forgiving convertion between unrelated data formats.
//!
//! - Support for versioning (allow to produce diffs between two versions of data, that can be later patched on demand).
//!
//!     Example: Game assets content difference for DLC or any episodic content usage; editor UI sending only changes to the game runtime to patch what actually changed in the world (instead of sending entire serialized object state).
//!
//! - Support for tagged intermediate data.
//!
//!     Example: Game asset stores data of type that is decided at runtime (associated tag gives hint what type its layout represents).
//!
//! ---
//!
//! ## Installation
//!
//! 1. Core crate with most important `Intermediate` and `ReflectIntermediate` types:
//!
//!     ```toml
//!     [dependencies]
//!     serde-intermediate = "*"
//!     ```
//!
//!     If you prefer to compile without `ReflectIntermediate` derive macro (`derive` feature adds derive macros and is enabled by default):
//!
//!     ```toml
//!     [dependencies]
//!     serde-intermediate = { version = "*", default-features = false }
//!     ```
//!
//! 1. Crate that adds support for tagged intermediate value (to embed tagged `Intermediate` in other serializable data with `TaggedIntermediate` type):
//!
//!     ```toml
//!     [dependencies]
//!     serde-tagged-intermediate = "*"
//!     ```
//!
//!     Same as with core crate, you can exclude `ReflectIntermediate` from compilation:
//!
//!     ```toml
//!     [dependencies]
//!     serde-tagged-intermediate = { version = "*", default-features = false }
//!     ```
//!
//! ---
//!
//! ## Examples
//!
//! Serialize/deserialize:
//!
//! ```rust
//! use std::time::SystemTime;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! enum Login {
//!     Email(String),
//!     SocialMedia{
//!         service: String,
//!         token: String,
//!         last_login: Option<SystemTime>,
//!     }
//! }
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! struct Person {
//!     // (first name, last name)
//!     name: (String, String),
//!     age: usize,
//!     login: Login,
//! }
//!
//! let data = Person {
//!     name: ("John".to_owned(), "Smith".to_owned()),
//!     age: 40,
//!     login: Login::Email("john.smith@gmail.com".to_owned()),
//! };
//! let serialized = serde_intermediate::to_intermediate(&data).unwrap();
//! let deserialized = serde_intermediate::from_intermediate(&serialized).unwrap();
//! assert_eq!(data, deserialized);
//! ```
//!
//! More elaborate problems and solutions:
//!
//! 1. Versioning (diff/patch) [(test_versioning)](https://github.com/PsichiX/serde-intermediate/blob/master/core/src/tests.rs#L440)
//! 1. Conversion between data layouts [(test_transform)](https://github.com/PsichiX/serde-intermediate/blob/master/core/src/tests.rs#L768)
//! 1. DLC / episodic content [(test_dlcs)](https://github.com/PsichiX/serde-intermediate/blob/master/core/src/tests.rs#L870)
//! 1. Data change communication between game and editor [(test_editor_communication)](https://github.com/PsichiX/serde-intermediate/blob/master/core/src/tests.rs#L1213)

#[cfg(test)]
mod tests;

pub mod de;
pub mod error;
pub mod reflect;
pub mod schema;
pub mod ser;
pub mod value;
pub mod versioning;

pub use crate::{
    de::{
        intermediate::{
            deserialize as from_intermediate, deserialize_as as from_intermediate_as,
            DeserializeMode,
        },
        object::deserialize as from_object,
        text::{from_str, from_str_as, intermediate_from_str},
    },
    error::Error,
    reflect::ReflectIntermediate,
    schema::{SchemaIdContainer, SchemaIntermediate, SchemaPackage},
    ser::{
        intermediate::serialize as to_intermediate,
        object::serialize as to_object,
        text::{to_string, to_string_compact, to_string_pretty, TextConfig, TextConfigStyle},
    },
    value::{intermediate::Intermediate, object::Object},
    versioning::*,
};

#[cfg(feature = "derive")]
pub use serde_intermediate_derive::*;
