#![cfg(test)]

use crate::{
    versioning::{Change, DiffOptimizationHint, DiffOptions},
    Intermediate,
};
use serde::{Deserialize, Serialize};
use std::collections::{
    hash_map::RandomState,
    {HashMap, HashSet},
};

macro_rules! map {
    ( $( $key:expr => $value:expr ),* ) => {{
        let mut result = HashMap::<_,_,RandomState>::default();
        $(
            result.insert($key, $value);
        )*
        result
    }}
}

macro_rules! set {
    ( $( $value:expr ),* ) => {{
        let mut result = HashSet::default();
        $(
            result.insert($value);
        )*
        result
    }}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct UnitStruct;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct NewTypeStruct(bool);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TupleStruct(bool, usize);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Enum {
    Unit,
    NewType(UnitStruct),
    Tuple(bool, usize),
    Struct { scalar: f32, text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Struct {
    pub bool_value: bool,
    pub i8_value: i8,
    pub i16_value: i16,
    pub i32_value: i32,
    pub i64_value: i64,
    pub i128_value: i128,
    pub u8_value: u8,
    pub u16_value: u16,
    pub u32_value: u32,
    pub u64_value: u64,
    pub u128_value: u128,
    pub f32_value: f32,
    pub f64_value: f64,
    pub char_value: char,
    pub string_value: String,
    pub tuple: (bool, usize),
    pub bytes: Vec<u8>,
    pub option: Option<UnitStruct>,
    pub list: Vec<usize>,
    pub set: HashSet<usize>,
    pub string_map: HashMap<String, usize>,
    pub integer_map: HashMap<usize, usize>,
    pub enum_value: Enum,
    pub new_type_struct: NewTypeStruct,
    pub tuple_struct: TupleStruct,
}

#[test]
fn test_simple() {
    let data = Struct {
        bool_value: true,
        i8_value: -1,
        i16_value: 2,
        i32_value: -3,
        i64_value: 4,
        i128_value: -5,
        u8_value: 6,
        u16_value: 7,
        u32_value: 8,
        u64_value: 9,
        u128_value: 10,
        f32_value: 1.1,
        f64_value: 1.2,
        char_value: '@',
        string_value: "hello".to_owned(),
        tuple: (false, 13),
        bytes: vec![14, 15, 16, 17, 18, 19],
        option: Some(UnitStruct),
        list: vec![20, 21, 23],
        set: set![20, 21, 23],
        string_map: map! {"a".to_owned() => 24,"b".to_owned() => 25},
        integer_map: map! {27 => 28,29 =>30},
        enum_value: Enum::Struct {
            scalar: 3.1,
            text: "world".to_owned(),
        },
        new_type_struct: NewTypeStruct(true),
        tuple_struct: TupleStruct(false, 32),
    };
    let serialized = crate::to_intermediate(&data).unwrap();
    let deserialized = crate::from_intermediate::<Struct>(&serialized).unwrap();
    assert_eq!(data, deserialized);
}

#[test]
fn test_size() {
    #[derive(Debug, Default, Serialize, Deserialize)]
    struct Foo([usize; 10]);

    #[derive(Debug, Default, Serialize, Deserialize)]
    struct Bar(Box<Foo>);

    let data = Bar::default();
    let serialized = crate::to_intermediate(&data).unwrap();
    assert_eq!(serialized.total_bytesize(), 728);
}

#[test]
fn test_general() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Foo {
        #[serde(default)]
        value: i32,
        #[serde(default)]
        list: Vec<String>,
    }

    let data = Foo {
        value: 42,
        list: vec!["a".to_owned(), "b".to_owned()],
    };
    let value = crate::to_intermediate(&data).unwrap();
    let serialized = serde_json::to_string_pretty(&value).unwrap();
    let expected = r#"{
  "value": 42,
  "list": [
    "a",
    "b"
  ]
}"#;
    assert_eq!(&serialized, expected);
    let serialized = serde_yaml::to_string(&value).unwrap();
    let expected = r#"---
value: 42
list:
  - a
  - b
"#;
    assert_eq!(&serialized, expected);
    let serialized = ron::to_string(&data).unwrap();
    let expected = r#"(value:42,list:["a","b"])"#;
    assert_eq!(&serialized, expected);
}

#[test]
fn test_struct() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Foo {
        #[serde(default)]
        value: i32,
        #[serde(default)]
        list: Vec<String>,
    }

    let data = Foo {
        value: 42,
        list: vec!["a".to_owned(), "b".to_owned()],
    };
    let serialized = crate::serialize(&data).unwrap();
    let deserialized = crate::deserialize::<Foo>(&serialized).unwrap();
    assert_eq!(data, deserialized);

    let serialized = Intermediate::struct_type().field("list", Intermediate::seq().item("c"));
    let deserialized = crate::deserialize::<Foo>(&serialized).unwrap();
    let data = Foo {
        value: 0,
        list: vec!["c".to_owned()],
    };
    assert_eq!(data, deserialized);

    let data = Foo {
        value: 42,
        list: vec!["a".to_owned(), "b".to_owned()],
    };
    let serialized = crate::serialize(&data).unwrap();
    let content = serde_json::to_string(&serialized).unwrap();
    let deserialized = serde_json::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = serde_yaml::to_string(&serialized).unwrap();
    let deserialized = serde_yaml::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = ron::to_string(&serialized).unwrap();
    let deserialized = ron::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = bincode::serialize(&serialized).unwrap();
    let deserialized = bincode::deserialize::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
}

#[test]
fn test_enum() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    enum Foo {
        A,
        B(bool, char),
        C {
            #[serde(default)]
            a: bool,
            #[serde(default)]
            b: char,
        },
    }

    let data = Foo::A;
    let serialized = crate::serialize(&data).unwrap();
    let deserialized = crate::deserialize::<Foo>(&serialized).unwrap();
    assert_eq!(data, deserialized);

    let data = Foo::B(true, '@');
    let serialized = crate::serialize(&data).unwrap();
    let deserialized = crate::deserialize::<Foo>(&serialized).unwrap();
    assert_eq!(data, deserialized);

    let data = Foo::C { a: true, b: '@' };
    let serialized = crate::serialize(&data).unwrap();
    let deserialized = crate::deserialize::<Foo>(&serialized).unwrap();
    assert_eq!(data, deserialized);

    let serialized = Intermediate::unit_variant("A", 0);
    let deserialized = crate::deserialize::<Foo>(&serialized).unwrap();
    let data = Foo::A;
    assert_eq!(data, deserialized);

    let serialized = Intermediate::tuple_variant("B", 1).item(true).item('@');
    let deserialized = crate::deserialize::<Foo>(&serialized).unwrap();
    let data = Foo::B(true, '@');
    assert_eq!(data, deserialized);

    let serialized = Intermediate::struct_variant("C", 2)
        .field("a", true)
        .field("b", '@');
    let deserialized = crate::deserialize::<Foo>(&serialized).unwrap();
    let data = Foo::C { a: true, b: '@' };
    assert_eq!(data, deserialized);

    let data = Foo::A;
    let serialized = crate::serialize(&data).unwrap();
    let content = serde_json::to_string(&serialized).unwrap();
    let deserialized = serde_json::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = serde_yaml::to_string(&serialized).unwrap();
    let deserialized = serde_yaml::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = ron::to_string(&serialized).unwrap();
    let deserialized = ron::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = bincode::serialize(&serialized).unwrap();
    let deserialized = bincode::deserialize::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);

    let data = Foo::B(true, '@');
    let serialized = crate::serialize(&data).unwrap();
    let content = serde_json::to_string(&serialized).unwrap();
    let deserialized = serde_json::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = serde_yaml::to_string(&serialized).unwrap();
    let deserialized = serde_yaml::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = ron::to_string(&serialized).unwrap();
    let deserialized = ron::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = bincode::serialize(&serialized).unwrap();
    let deserialized = bincode::deserialize::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);

    let data = Foo::C { a: true, b: '@' };
    let serialized = crate::serialize(&data).unwrap();
    let content = serde_json::to_string(&serialized).unwrap();
    let deserialized = serde_json::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = serde_yaml::to_string(&serialized).unwrap();
    let deserialized = serde_yaml::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = ron::to_string(&serialized).unwrap();
    let deserialized = ron::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = bincode::serialize(&serialized).unwrap();
    let deserialized = bincode::deserialize::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
}

#[test]
fn test_migration() {
    #[derive(Debug, Serialize, Deserialize)]
    struct VersionA {
        a: usize,
        b: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct VersionB {
        a: isize,
        b: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct VersionC {
        b: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct VersionD {
        b: String,
        #[serde(default)]
        d: Vec<usize>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct VersionE {
        #[serde(default)]
        d: HashSet<usize>,
    }

    fn migrate<R: serde::de::DeserializeOwned, T: Serialize>(from: T) -> R {
        let content = serde_json::to_string(&from).expect("Could not serialzie into JSON");
        let intermediate = serde_json::from_str::<Intermediate>(&content)
            .expect("Could not deserialize from JSON");
        crate::deserialize(&intermediate).expect("Could not deserialize from intermediate")
    }

    let data = VersionA {
        a: 42,
        b: "text".to_owned(),
    };
    println!("{:#?}", data);
    let data = migrate::<VersionB, _>(data);
    println!("{:#?}", data);
    let data = migrate::<VersionC, _>(data);
    println!("{:#?}", data);
    let data = migrate::<VersionD, _>(data);
    println!("{:#?}", data);
    let data = migrate::<VersionE, _>(data);
    println!("{:#?}", data);
}

#[test]
fn test_seq_diff() {
    let data = vec!["a", "b", "c"];
    let value = crate::to_intermediate(&data).unwrap();
    let provided = Change::sequence_difference(value.as_seq().unwrap(), value.as_seq().unwrap());
    assert_eq!(provided, vec![]);

    let data = vec!["a", "b", "c", "d"];
    let prev = crate::to_intermediate(&data).unwrap();
    let data = vec!["e", "f", "g", "h", "c", "d"];
    let next = crate::to_intermediate(&data).unwrap();
    let provided = Change::sequence_difference(prev.as_seq().unwrap(), next.as_seq().unwrap());
    let expected = vec![
        (0, Change::Added("e".into())),
        (1, Change::Added("f".into())),
        (2, Change::Changed("g".into())),
        (3, Change::Changed("h".into())),
    ];
    assert_eq!(provided, expected);

    let data = vec!["a", "b", "c", "d"];
    let prev = crate::to_intermediate(&data).unwrap();
    let data = vec!["e", "a", "b", "c", "f"];
    let next = crate::to_intermediate(&data).unwrap();
    let provided = Change::sequence_difference(prev.as_seq().unwrap(), next.as_seq().unwrap());
    let expected = vec![
        (0, Change::Added("e".into())),
        (4, Change::Changed("f".into())),
    ];
    assert_eq!(provided, expected);
}

#[test]
fn test_versioning() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Foo {
        #[serde(default)]
        map: HashMap<String, usize>,
        #[serde(default)]
        list: Vec<String>,
    }

    let prev = Option::<usize>::None;
    let prev = crate::to_intermediate(&prev).unwrap();
    let next = Option::<usize>::Some(42);
    let next = crate::to_intermediate(&next).unwrap();
    let diff = Change::difference(&prev, &next, &Default::default());
    let patched = diff.patch(&prev).unwrap().unwrap();
    assert_eq!(patched, next);

    let prev = Result::<usize, bool>::Ok(42);
    let prev = crate::to_intermediate(&prev).unwrap();
    let next = Result::<usize, bool>::Err(true);
    let next = crate::to_intermediate(&next).unwrap();
    let diff = Change::difference(&prev, &next, &Default::default());
    let patched = diff.patch(&prev).unwrap().unwrap();
    assert_eq!(patched, next);

    let prev = Foo {
        map: map! {"answer".to_owned() => 42},
        list: vec!["hello".to_owned()],
    };
    let prev = crate::to_intermediate(&prev).unwrap();
    let next = Foo {
        map: map! {"answer".to_owned() => 0},
        list: vec!["hello".to_owned(), "world".to_owned()],
    };
    let next = crate::to_intermediate(&next).unwrap();
    let diff = Change::difference(&prev, &next, &Default::default());
    let patched = diff.patch(&prev).unwrap().unwrap();
    assert_eq!(patched, next);

    let source = Intermediate::default();
    let change = Change::Same;
    let patched = change
        .patch(&source)
        .expect("Could not patch source")
        .unwrap();
    assert_eq!(patched, source);

    let source = Intermediate::default();
    let change = Change::Removed;
    let patched = change.patch(&source).expect("Could not patch source");
    assert_eq!(patched, None);

    let source = Intermediate::seq().item(1).item(2).item(3);
    let expected = Intermediate::seq().item(1).item(2);
    let change = Change::partial_seq()
        .partial_seq_item(1, Change::Removed)
        .partial_seq_item(1, Change::Changed(2.into()));
    let patched = change
        .patch(&source)
        .expect("Could not patch source")
        .unwrap();
    assert_eq!(patched, expected);

    let source = Intermediate::tuple().item(1).item("hello");
    let expected = Intermediate::tuple().item(1).item("world");
    let change = Change::PartialSeq(vec![
        (0, Change::Same),
        (1, Change::Changed("world".into())),
    ]);
    let patched = change
        .patch(&source)
        .expect("Could not patch source")
        .unwrap();
    assert_eq!(patched, expected);

    let source = Intermediate::struct_type().field("hey", "hello");
    let expected = Intermediate::struct_type().field("hi", "hello");
    let change = Change::PartialStruct(vec![
        ("hey".to_owned(), Change::Removed),
        ("hi".to_owned(), Change::Added("hello".into())),
    ]);
    let patched = change
        .patch(&source)
        .expect("Could not patch source")
        .unwrap();
    assert_eq!(patched, expected);

    let prev = Struct {
        bool_value: true,
        i8_value: -1,
        i16_value: 2,
        i32_value: -3,
        i64_value: 4,
        i128_value: -5,
        u8_value: 6,
        u16_value: 7,
        u32_value: 8,
        u64_value: 9,
        u128_value: 10,
        f32_value: 1.1,
        f64_value: 1.2,
        char_value: '@',
        string_value: "hello".to_owned(),
        tuple: (false, 13),
        bytes: vec![14, 15, 16, 17, 18, 19],
        option: Some(UnitStruct),
        list: vec![20, 21, 23],
        set: set![20, 21, 23],
        string_map: map! {"a".to_owned() => 24,"b".to_owned() => 25},
        integer_map: map! {27 => 28,29 =>30},
        enum_value: Enum::Struct {
            scalar: 3.1,
            text: "world".to_owned(),
        },
        new_type_struct: NewTypeStruct(true),
        tuple_struct: TupleStruct(false, 32),
    };
    let next = Struct {
        bool_value: false,
        i8_value: 1,
        i16_value: 2,
        i32_value: 3,
        i64_value: 4,
        i128_value: 5,
        u8_value: 6,
        u16_value: 7,
        u32_value: 8,
        u64_value: 9,
        u128_value: 10,
        f32_value: 1.1,
        f64_value: 1.2,
        char_value: '@',
        string_value: "hello".to_owned(),
        tuple: (false, 13),
        bytes: vec![14, 18, 17],
        option: Some(UnitStruct),
        list: vec![20, 21, 23, 21, 23],
        set: set![20],
        string_map: map! {"c".to_owned() => 24},
        integer_map: map! {27 => 28},
        enum_value: Enum::Struct {
            scalar: 3.1,
            text: "erm".to_owned(),
        },
        new_type_struct: NewTypeStruct(true),
        tuple_struct: TupleStruct(false, 42),
    };
    let diff = Change::data_difference(&prev, &next, &Default::default()).unwrap();
    let patched = diff.data_patch(&prev).unwrap().unwrap();
    assert_eq!(next, patched);

    let prev = crate::to_intermediate(&prev).unwrap();
    let next = crate::to_intermediate(&next).unwrap();
    let diff = Change::difference(
        &prev,
        &next,
        &DiffOptions::default().optimization_hint(DiffOptimizationHint::SizePercentage(0.5)),
    );
    let patched = diff.patch(&prev).expect("Could not patch source").unwrap();
    assert_eq!(patched, next);

    let base = Foo {
        map: map!("a".to_owned() => 1, "b".to_owned() => 2),
        list: vec!["hello".to_owned(), "world".to_owned()],
    };
    let patch_a = Foo {
        map: map!("a".to_owned() => 1),
        list: vec!["hello".to_owned(), "foo".to_owned()],
    };
    let patch_b = Foo {
        map: map!("a".to_owned() => 42, "b".to_owned() => 2),
        list: vec!["foo".to_owned()],
    };
    let change_a = Change::data_difference(&base, &patch_a, &Default::default()).unwrap();
    let change_b = Change::data_difference(&base, &patch_b, &Default::default()).unwrap();
    {
        let patched = change_a.data_patch(&base).unwrap().unwrap();
        assert_eq!(patched, patch_a);
    }
    {
        let patched = change_b.data_patch(&base).unwrap().unwrap();
        assert_eq!(patched, patch_b);
    }
    {
        let patched = change_a.data_patch(&base).unwrap().unwrap();
        let patched = change_b.data_patch(&patched).unwrap().unwrap();
        let expected = Foo {
            map: map!("a".to_owned() => 42),
            list: vec!["foo".to_owned()],
        };
        assert_eq!(patched, expected);
    }
    {
        let patched = change_b.data_patch(&base).unwrap().unwrap();
        let patched = change_a.data_patch(&patched).unwrap().unwrap();
        let expected = Foo {
            map: map!("a".to_owned() => 42),
            list: vec!["foo".to_owned()],
        };
        assert_eq!(patched, expected);
    }
}

#[test]
fn test_transform() {
    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct Foo {
        #[serde(default)]
        a: usize,
        #[serde(default)]
        b: usize,
    }

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct Bar {
        #[serde(default)]
        a: f32,
        #[serde(default)]
        b: i32,
        #[serde(default)]
        c: String,
    }

    let data = map! { "a".to_owned() => 1usize, "b".to_owned() => 2usize};
    let serialized = crate::to_intermediate(&data).unwrap();
    let deserialized = crate::from_intermediate::<Foo>(&serialized).unwrap();
    let expected = Foo { a: 1, b: 2 };
    assert_eq!(deserialized, expected);

    let data = Foo { a: 1, b: 2 };
    let serialized = crate::to_intermediate(&data).unwrap();
    let deserialized = crate::from_intermediate::<Bar>(&serialized).unwrap();
    let expected = Bar {
        a: 1.0,
        b: 2,
        c: "".to_owned(),
    };
    assert_eq!(deserialized, expected);
}

#[test]
fn test_debug() {
    let data = Struct {
        bool_value: true,
        i8_value: -1,
        i16_value: 2,
        i32_value: -3,
        i64_value: 4,
        i128_value: -5,
        u8_value: 6,
        u16_value: 7,
        u32_value: 8,
        u64_value: 9,
        u128_value: 10,
        f32_value: 1.1,
        f64_value: 1.2,
        char_value: '@',
        string_value: "hello".to_owned(),
        tuple: (false, 13),
        bytes: vec![14, 15, 16, 17, 18, 19],
        option: Some(UnitStruct),
        list: vec![20, 21, 23],
        set: set![20, 21, 23],
        string_map: map! {"a".to_owned() => 24,"b".to_owned() => 25},
        integer_map: map! {27 => 28,29 =>30},
        enum_value: Enum::Struct {
            scalar: 3.1,
            text: "world".to_owned(),
        },
        new_type_struct: NewTypeStruct(true),
        tuple_struct: TupleStruct(false, 32),
    };
    let serialized = crate::to_intermediate(&data).unwrap();
    println!("Debug: {:?}", serialized);
}

#[test]
fn test_container() {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct Foo {
        a: Intermediate,
        b: Intermediate,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct Bar {
        c: f32,
        d: usize,
    }

    let a = Bar { c: 1.2, d: 3 };
    let b = "hello world!".to_owned();
    let data = Foo {
        a: crate::to_intermediate(&a).unwrap(),
        b: crate::to_intermediate(&b).unwrap(),
    };
    let serialized = serde_json::to_string_pretty(&data).unwrap();
    let deserialized = serde_json::from_str::<Foo>(&serialized).unwrap();
    assert_eq!(crate::from_intermediate::<Bar>(&deserialized.a).unwrap(), a);
    assert_eq!(
        crate::from_intermediate::<String>(&deserialized.b).unwrap(),
        b
    );
}
