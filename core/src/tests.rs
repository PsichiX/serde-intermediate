use crate::{
    versioning::{Change, DiffOptimizationHint, DiffOptions},
    Intermediate, Object, ReflectIntermediate, SchemaIntermediate, TextConfig, TextConfigStyle,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    cell::Cell,
    collections::{
        hash_map::RandomState,
        {HashMap, HashSet},
    },
    path::PathBuf,
    sync::mpsc::channel,
    sync::Mutex,
};

use crate as serde_intermediate;

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

fn try_until<F, T>(mut tries: usize, mut provide: F, expected: T)
where
    F: FnMut() -> T,
    T: PartialEq + std::fmt::Debug,
{
    tries = tries.max(1);
    let mut last = None;
    for _ in 0..tries {
        let provided = provide();
        if provided == expected {
            return;
        } else {
            last = Some(provided);
        }
    }
    panic!(
        "Could not provide {:#?} in {} tries! Last provided value: {:#?}",
        expected, tries, last
    );
}

/// Unit struct.
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, ReflectIntermediate, SchemaIntermediate,
)]
struct UnitStruct;

/// New type struct.
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, ReflectIntermediate, SchemaIntermediate,
)]
struct NewTypeStruct(
    /// Bool value.
    bool,
);

/// Tuple struct.
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, ReflectIntermediate, SchemaIntermediate,
)]
struct TupleStruct(bool, usize);

/// Enum.
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, ReflectIntermediate, SchemaIntermediate,
)]
enum Enum {
    Unit,
    NewType(#[schema_intermediate(package)] UnitStruct),
    Tuple(bool, usize),
    Struct { scalar: f32, text: String },
}

/// Struct.
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, ReflectIntermediate, SchemaIntermediate,
)]
struct Struct {
    bool_value: bool,
    i8_value: i8,
    i16_value: i16,
    i32_value: i32,
    i64_value: i64,
    i128_value: i128,
    u8_value: u8,
    u16_value: u16,
    u32_value: u32,
    u64_value: u64,
    u128_value: u128,
    f32_value: f32,
    f64_value: f64,
    char_value: char,
    string_value: String,
    tuple: (bool, usize),
    bytes: Vec<u8>,
    #[schema_intermediate(package_traverse(UnitStruct))]
    option: Option<UnitStruct>,
    list: Vec<usize>,
    set: HashSet<usize>,
    string_map: HashMap<String, usize>,
    integer_map: HashMap<usize, usize>,
    #[schema_intermediate(package)]
    enum_value: Enum,
    #[schema_intermediate(package)]
    new_type_struct: NewTypeStruct,
    #[schema_intermediate(package)]
    tuple_struct: TupleStruct,
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
    assert_eq!(serialized.total_bytesize(), 832);
}

#[test]
fn test_object() {
    #[derive(
        Debug, Clone, Serialize, Deserialize, PartialEq, ReflectIntermediate, SchemaIntermediate,
    )]
    struct Foo {
        bool_value: bool,
        i8_value: i8,
        i16_value: i16,
        i32_value: i32,
        i64_value: i64,
        u8_value: u8,
        u16_value: u16,
        u32_value: u32,
        u64_value: u64,
        f32_value: f32,
        f64_value: f64,
        char_value: char,
        string_value: String,
        tuple: (bool, usize),
        bytes: Vec<u8>,
        option: Option<UnitStruct>,
        list: Vec<usize>,
        set: HashSet<usize>,
        string_map: HashMap<String, usize>,
        integer_map: HashMap<usize, usize>,
        enum_value: Enum,
        new_type_struct: NewTypeStruct,
        tuple_struct: TupleStruct,
    }

    let data = Foo {
        bool_value: true,
        i8_value: -1,
        i16_value: 2,
        i32_value: -3,
        i64_value: 4,
        u8_value: 6,
        u16_value: 7,
        u32_value: 8,
        u64_value: 9,
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
    let serialized = crate::to_object(&data).unwrap();
    let deserialized = crate::from_object::<Foo>(&serialized).unwrap();
    assert_eq!(data, deserialized);
    let serialized = crate::to_string_pretty(&serialized).unwrap();
    let deserialized = crate::from_str(&serialized).unwrap();
    assert_eq!(data, deserialized);
    let serialized = crate::to_string_compact(&data).unwrap();
    let deserialized = crate::from_str(&serialized).unwrap();
    assert_eq!(data, deserialized);
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
    // TODO: uncomment when struct-as-map serialization issue gets fixed!
    // let serialized = crate::to_intermediate(&data).unwrap();
    // let deserialized = crate::from_intermediate::<Intermediate>(&serialized).unwrap();
    // assert_eq!(serialized, deserialized);
    let serialized = crate::to_object(&data).unwrap();
    let deserialized = crate::from_object::<Object>(&serialized).unwrap();
    assert_eq!(serialized, deserialized);
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
    let expected = r#"value: 42
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
    let serialized = crate::to_intermediate(&data).unwrap();
    let deserialized = crate::from_intermediate::<Foo>(&serialized).unwrap();
    assert_eq!(data, deserialized);

    let serialized = Intermediate::struct_type().field("list", Intermediate::seq().item("c"));
    let deserialized = crate::from_intermediate::<Foo>(&serialized).unwrap();
    let data = Foo {
        value: 0,
        list: vec!["c".to_owned()],
    };
    assert_eq!(data, deserialized);

    let data = Foo {
        value: 42,
        list: vec!["a".to_owned(), "b".to_owned()],
    };
    let serialized = crate::to_intermediate(&data).unwrap();
    let content = serde_json::to_string(&serialized).unwrap();
    let deserialized = serde_json::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = serde_yaml::to_string(&serialized).unwrap();
    let deserialized = serde_yaml::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = ron::to_string(&serialized).unwrap();
    let deserialized = ron::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = crate::to_string_pretty(&serialized).unwrap();
    let deserialized = crate::from_str::<Foo>(&content).unwrap();
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
    let serialized = crate::to_intermediate(&data).unwrap();
    let deserialized = crate::from_intermediate::<Foo>(&serialized).unwrap();
    assert_eq!(data, deserialized);

    let data = Foo::B(true, '@');
    let serialized = crate::to_intermediate(&data).unwrap();
    let deserialized = crate::from_intermediate::<Foo>(&serialized).unwrap();
    assert_eq!(data, deserialized);

    let data = Foo::C { a: true, b: '@' };
    let serialized = crate::to_intermediate(&data).unwrap();
    let deserialized = crate::from_intermediate::<Foo>(&serialized).unwrap();
    assert_eq!(data, deserialized);

    let serialized = Intermediate::unit_variant("A");
    let deserialized = crate::from_intermediate::<Foo>(&serialized).unwrap();
    let data = Foo::A;
    assert_eq!(data, deserialized);

    let serialized = Intermediate::tuple_variant("B").item(true).item('@');
    let deserialized = crate::from_intermediate::<Foo>(&serialized).unwrap();
    let data = Foo::B(true, '@');
    assert_eq!(data, deserialized);

    let serialized = Intermediate::struct_variant("C")
        .field("a", true)
        .field("b", '@');
    let deserialized = crate::from_intermediate::<Foo>(&serialized).unwrap();
    let data = Foo::C { a: true, b: '@' };
    assert_eq!(data, deserialized);

    let data = Foo::A;
    let serialized = crate::to_intermediate(&data).unwrap();
    let content = serde_json::to_string(&serialized).unwrap();
    let deserialized = serde_json::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = serde_yaml::to_string(&serialized).unwrap();
    let deserialized = serde_yaml::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = ron::to_string(&serialized).unwrap();
    let deserialized = ron::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = crate::to_string_pretty(&serialized).unwrap();
    println!("{:?}", content);
    let deserialized = crate::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);

    let data = Foo::B(true, '@');
    let serialized = crate::to_intermediate(&data).unwrap();
    let content = serde_json::to_string(&serialized).unwrap();
    let deserialized = serde_json::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = serde_yaml::to_string(&serialized).unwrap();
    let deserialized = serde_yaml::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = ron::to_string(&serialized).unwrap();
    let deserialized = ron::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = crate::to_string_pretty(&serialized).unwrap();
    let deserialized = crate::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);

    let data = Foo::C { a: true, b: '@' };
    let serialized = crate::to_intermediate(&data).unwrap();
    let content = serde_json::to_string(&serialized).unwrap();
    let deserialized = serde_json::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = serde_yaml::to_string(&serialized).unwrap();
    let deserialized = serde_yaml::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = ron::to_string(&serialized).unwrap();
    let deserialized = ron::from_str::<Foo>(&content).unwrap();
    assert_eq!(data, deserialized);
    let content = crate::to_string_pretty(&serialized).unwrap();
    let deserialized = crate::from_str::<Foo>(&content).unwrap();
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
        crate::from_intermediate(&intermediate).expect("Could not deserialize from intermediate")
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
    let provided = Change::sequence_difference(
        value.as_seq().unwrap(),
        value.as_seq().unwrap(),
        &Default::default(),
    );
    assert_eq!(provided, vec![]);

    let data = vec!["a", "b", "c", "d"];
    let prev = crate::to_intermediate(&data).unwrap();
    let data = vec!["e", "f", "g", "h", "c", "d"];
    let next = crate::to_intermediate(&data).unwrap();
    let provided = Change::sequence_difference(
        prev.as_seq().unwrap(),
        next.as_seq().unwrap(),
        &Default::default(),
    );
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
    let provided = Change::sequence_difference(
        prev.as_seq().unwrap(),
        next.as_seq().unwrap(),
        &Default::default(),
    );
    let expected = vec![
        (0, Change::Added("e".into())),
        (4, Change::Changed("f".into())),
    ];
    assert_eq!(provided, expected);
}

#[test]
fn test_versioning() {
    #[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, ReflectIntermediate)]
    struct Foo {
        #[serde(default)]
        map: HashMap<String, usize>,
        #[serde(default)]
        list: Vec<String>,
    }

    #[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, ReflectIntermediate)]
    struct Wrapper {
        v: bool,
    }

    impl Wrapper {
        fn new(v: bool) -> Self {
            Self { v }
        }
    }

    let mut source = Option::<usize>::None;
    let prev = crate::to_intermediate(&source).unwrap();
    let next = Option::<usize>::Some(42);
    let next = crate::to_intermediate(&next).unwrap();
    let diff = Change::difference(&prev, &next, &Default::default());
    let patched = diff.patch(&prev).unwrap().unwrap();
    assert_eq!(patched, next);
    source.patch_change(&diff);
    let target = crate::from_intermediate(&next).unwrap();
    assert_eq!(source, target);

    let mut source = Some(Wrapper::new(false));
    let prev = crate::to_intermediate(&source).unwrap();
    let next = Some(Wrapper::new(true));
    let next = crate::to_intermediate(&next).unwrap();
    let diff = Change::difference(&prev, &next, &Default::default());
    let patched = diff.patch(&prev).unwrap().unwrap();
    assert_eq!(patched, next);
    source.patch_change(&diff);
    let target = crate::from_intermediate(&next).unwrap();
    assert_eq!(source, target);

    let mut source = Result::<usize, bool>::Ok(42);
    let prev = crate::to_intermediate(&source).unwrap();
    let next = Result::<usize, bool>::Err(true);
    let next = crate::to_intermediate(&next).unwrap();
    let diff = Change::difference(&prev, &next, &Default::default());
    let patched = diff.patch(&prev).unwrap().unwrap();
    assert_eq!(patched, next);
    source.patch_change(&diff);
    let target = crate::from_intermediate(&next).unwrap();
    assert_eq!(source, target);

    let mut source = Result::<Wrapper, ()>::Ok(Wrapper::new(false));
    let prev = crate::to_intermediate(&source).unwrap();
    let next = Result::<Wrapper, ()>::Ok(Wrapper::new(true));
    let next = crate::to_intermediate(&next).unwrap();
    let diff = Change::difference(&prev, &next, &Default::default());
    let patched = diff.patch(&prev).unwrap().unwrap();
    assert_eq!(patched, next);
    source.patch_change(&diff);
    let target = crate::from_intermediate(&next).unwrap();
    assert_eq!(source, target);

    let mut source = Cell::new(false);
    let prev = crate::to_intermediate(&source).unwrap();
    let next = Cell::new(true);
    let next = crate::to_intermediate(&next).unwrap();
    let diff = Change::difference(&prev, &next, &Default::default());
    let patched = diff.patch(&prev).unwrap().unwrap();
    assert_eq!(patched, next);
    source.patch_change(&diff);
    let target = crate::from_intermediate(&next).unwrap();
    assert_eq!(source, target);

    let mut source = Mutex::new(false);
    let prev = crate::to_intermediate(&source).unwrap();
    let next = Mutex::new(true);
    let next = crate::to_intermediate(&next).unwrap();
    let diff = Change::difference(&prev, &next, &Default::default());
    let patched = diff.patch(&prev).unwrap().unwrap();
    assert_eq!(patched, next);
    source.patch_change(&diff);
    let target = crate::from_intermediate::<Mutex<bool>>(&next).unwrap();
    {
        let source = source.lock().unwrap();
        let target = target.lock().unwrap();
        assert_eq!(*source, *target);
    }

    let mut source = 0..5;
    let prev = crate::to_intermediate(&source).unwrap();
    let next = 5..10;
    let next = crate::to_intermediate(&next).unwrap();
    let diff = Change::difference(&prev, &next, &Default::default());
    let patched = diff.patch(&prev).unwrap().unwrap();
    assert_eq!(patched, next);
    source.patch_change(&diff);
    let target = crate::from_intermediate(&next).unwrap();
    assert_eq!(source, target);

    let mut source = 0..=5;
    let prev = crate::to_intermediate(&source).unwrap();
    let next = 5..=10;
    let next = crate::to_intermediate(&next).unwrap();
    let diff = Change::difference(&prev, &next, &Default::default());
    let patched = diff.patch(&prev).unwrap().unwrap();
    assert_eq!(patched, next);
    source.patch_change(&diff);
    let target = crate::from_intermediate(&next).unwrap();
    assert_eq!(source, target);

    let mut prev = vec![Foo {
        map: map! {"answer".to_owned() => 0},
        list: vec!["hello".to_owned()],
    }];
    let next = vec![Foo {
        map: map! {"answer".to_owned() => 42},
        list: vec!["hello".to_owned()],
    }];
    let diff = Change::data_difference(&prev, &next, &Default::default()).unwrap();
    prev.patch_change(&diff);
    assert_eq!(prev, next);

    let mut source = Foo {
        map: map! {"answer".to_owned() => 42},
        list: vec!["hello".to_owned()],
    };
    let prev = crate::to_intermediate(&source).unwrap();
    let next = Foo {
        map: map! {"answer".to_owned() => 0},
        list: vec!["hello".to_owned(), "world".to_owned()],
    };
    let next = crate::to_intermediate(&next).unwrap();
    let diff = Change::difference(&prev, &next, &Default::default());
    let patched = diff.patch(&prev).unwrap().unwrap();
    assert_eq!(patched, next);
    source.patch_change(&diff);
    let target = crate::from_intermediate(&next).unwrap();
    assert_eq!(source, target);

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

    let mut source = Intermediate::seq().item(1).item(2).item(3);
    let expected = Intermediate::seq().item(1).item(2);
    let change = Change::partial_seq()
        .partial_seq_item(1, Change::Removed)
        .partial_seq_item(1, Change::Changed(2.into()));
    let patched = change
        .patch(&source)
        .expect("Could not patch source")
        .unwrap();
    assert_eq!(patched, expected);
    source.patch_change(&change);
    assert_eq!(source, expected);

    let mut source = Intermediate::tuple().item(1).item("hello");
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
    source.patch_change(&change);
    assert_eq!(source, expected);

    let mut source = Intermediate::struct_type().field("hey", "hello");
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
    source.patch_change(&change);
    assert_eq!(source, expected);

    let mut prev = Struct {
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
    prev.patch_change(&diff);
    assert_eq!(prev, next);

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
        let mut base = base.to_owned();
        base.patch_change(&change_a);
        assert_eq!(base, patch_a);
    }
    {
        let patched = change_b.data_patch(&base).unwrap().unwrap();
        assert_eq!(patched, patch_b);
        let mut base = base.to_owned();
        base.patch_change(&change_b);
        assert_eq!(base, patch_b);
    }
    {
        let patched = change_a.data_patch(&base).unwrap().unwrap();
        let patched = change_b.data_patch(&patched).unwrap().unwrap();
        let expected = Foo {
            map: map!("a".to_owned() => 42),
            list: vec!["foo".to_owned()],
        };
        assert_eq!(patched, expected);
        let mut base = base.to_owned();
        base.patch_change(&change_a);
        base.patch_change(&change_b);
        assert_eq!(base, expected);
    }
    {
        let patched = change_b.data_patch(&base).unwrap().unwrap();
        let patched = change_a.data_patch(&patched).unwrap().unwrap();
        let expected = Foo {
            map: map!("a".to_owned() => 42),
            list: vec!["foo".to_owned()],
        };
        assert_eq!(patched, expected);
        let mut base = base.to_owned();
        base.patch_change(&change_b);
        base.patch_change(&change_a);
        assert_eq!(base, expected);
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

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct Zee {
        a: Object,
        b: Object,
    }

    let a = Bar { c: 1.2, d: 3 };
    let b = "hello world!".to_owned();
    let data = Zee {
        a: crate::to_object(&a).unwrap(),
        b: crate::to_object(&b).unwrap(),
    };
    let serialized = ron::to_string(&data).unwrap();
    let deserialized = ron::from_str::<Zee>(&serialized).unwrap();
    assert_eq!(crate::from_object::<Bar>(&deserialized.a).unwrap(), a);
    assert_eq!(crate::from_object::<String>(&deserialized.b).unwrap(), b);
}

#[test]
fn test_dlcs() {
    #[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, ReflectIntermediate)]
    enum Primitive {
        #[default]
        None,
        Sprite {
            file: PathBuf,
            size: (f32, f32),
            pivot: (f32, f32),
        },
        Script {
            file: PathBuf,
            properties: HashMap<String, Intermediate>,
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ReflectIntermediate)]
    enum SceneContent {
        File(PathBuf),
        Children(HashMap<String, SceneNode>),
    }

    impl Default for SceneContent {
        fn default() -> Self {
            Self::Children(Default::default())
        }
    }

    impl SceneContent {
        fn with_child(mut self, name: impl ToString, node: SceneNode) -> Self {
            if let Self::Children(children) = &mut self {
                children.insert(name.to_string(), node);
            }
            self
        }
    }

    #[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, ReflectIntermediate)]
    struct SceneNode {
        primitive: Primitive,
        position: (f32, f32),
        content: SceneContent,
    }

    impl SceneNode {
        fn with_primitive(mut self, primitive: Primitive) -> Self {
            self.primitive = primitive;
            self
        }

        fn with_position(mut self, x: f32, y: f32) -> Self {
            self.position = (x, y);
            self
        }

        fn with_content(mut self, content: SceneContent) -> Self {
            self.content = content;
            self
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ReflectIntermediate)]
    enum Asset {
        Scene(SceneNode),
        Script(String),
    }

    #[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
    struct Package {
        base: HashMap<PathBuf, Asset>,
        changes: HashMap<PathBuf, Change>,
    }

    impl Package {
        fn with_base(mut self, path: impl Into<PathBuf>, asset: Asset) -> Self {
            self.base.insert(path.into(), asset);
            self
        }

        fn with_change(mut self, path: impl Into<PathBuf>, change: Change) -> Self {
            self.changes.insert(path.into(), change);
            self
        }

        fn patch(&self, other: &Self) -> Self {
            let mut base = self
                .base
                .iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .chain(other.base.iter().map(|(k, v)| (k.to_owned(), v.to_owned())))
                .collect::<HashMap<_, _>>();
            for (path, change) in &self.changes {
                if let Some(asset) = base.get_mut(path) {
                    asset.patch_change(change);
                }
            }
            for (path, change) in &other.changes {
                if let Some(asset) = base.get_mut(path) {
                    asset.patch_change(change);
                }
            }
            Self {
                base,
                ..Default::default()
            }
        }
    }

    let level_a = Asset::Scene(
        SceneNode::default().with_content(
            SceneContent::default()
                .with_child(
                    "background",
                    SceneNode::default().with_primitive(Primitive::Sprite {
                        file: "summer.png".into(),
                        size: (1024.0, 1024.0),
                        pivot: Default::default(),
                    }),
                )
                .with_child(
                    "player",
                    SceneNode::default()
                        .with_primitive(Primitive::Script {
                            file: "player.lua".into(),
                            properties: Default::default(),
                        })
                        .with_position(100.0, -200.0)
                        .with_content(SceneContent::default().with_child(
                            "sprite",
                            SceneNode::default().with_primitive(Primitive::Sprite {
                                file: "player.png".into(),
                                size: (32.0, 32.0),
                                pivot: (0.5, 0.5),
                            }),
                        )),
                ),
        ),
    );
    let package_a = Package::default().with_base("level.scn", level_a.to_owned());

    let player_b = Asset::Scene(
        SceneNode::default()
            .with_primitive(Primitive::Script {
                file: "player.lua".into(),
                properties: Default::default(),
            })
            .with_content(SceneContent::default().with_child(
                "sprite",
                SceneNode::default().with_primitive(Primitive::Sprite {
                    file: "player.png".into(),
                    size: (32.0, 32.0),
                    pivot: (0.5, 0.5),
                }),
            )),
    );
    let level_b = Asset::Scene(
        SceneNode::default().with_content(
            SceneContent::default()
                .with_child(
                    "background",
                    SceneNode::default().with_primitive(Primitive::Sprite {
                        file: "winter.png".into(),
                        size: (1024.0, 1024.0),
                        pivot: Default::default(),
                    }),
                )
                .with_child(
                    "player",
                    SceneNode::default()
                        .with_position(100.0, 200.0)
                        .with_content(SceneContent::File("player.scn".into())),
                ),
        ),
    );
    let level_b_diff = Change::data_difference(&level_a, &level_b, &Default::default()).unwrap();
    let package_b = Package::default()
        .with_base("player.scn", player_b.to_owned())
        .with_change("level.scn", level_b_diff);

    let enemy_c = Asset::Scene(
        SceneNode::default()
            .with_primitive(Primitive::Script {
                file: "enemy.lua".into(),
                properties: Default::default(),
            })
            .with_content(SceneContent::default().with_child(
                "sprite",
                SceneNode::default().with_primitive(Primitive::Sprite {
                    file: "enemy.png".into(),
                    size: (64.0, 64.0),
                    pivot: (0.5, 0.5),
                }),
            )),
    );
    let level_c = Asset::Scene(
        SceneNode::default().with_content(
            SceneContent::default()
                .with_child(
                    "background",
                    SceneNode::default().with_primitive(Primitive::Sprite {
                        file: "winter.png".into(),
                        size: (1024.0, 1024.0),
                        pivot: Default::default(),
                    }),
                )
                .with_child(
                    "player",
                    SceneNode::default()
                        .with_position(100.0, 200.0)
                        .with_content(SceneContent::File("player.scn".into())),
                )
                .with_child(
                    "enemy",
                    SceneNode::default()
                        .with_position(900.0, 800.0)
                        .with_content(SceneContent::File("enemy.scn".into())),
                ),
        ),
    );
    let level_c_diff = Change::data_difference(&level_b, &level_c, &Default::default()).unwrap();
    let package_c = Package::default()
        .with_base("player.scn", player_b.to_owned())
        .with_base("enemy.scn", enemy_c.to_owned())
        .with_change("level.scn", level_c_diff);

    let boss_d = Asset::Scene(
        SceneNode::default()
            .with_primitive(Primitive::Script {
                file: "boss.lua".into(),
                properties: Default::default(),
            })
            .with_content(SceneContent::default().with_child(
                "sprite",
                SceneNode::default().with_primitive(Primitive::Sprite {
                    file: "boss.png".into(),
                    size: (128.0, 128.0),
                    pivot: (0.5, 0.5),
                }),
            )),
    );
    let level_d = Asset::Scene(
        SceneNode::default().with_content(
            SceneContent::default()
                .with_child(
                    "background",
                    SceneNode::default().with_primitive(Primitive::Sprite {
                        file: "winter.png".into(),
                        size: (1024.0, 1024.0),
                        pivot: Default::default(),
                    }),
                )
                .with_child(
                    "player",
                    SceneNode::default()
                        .with_position(100.0, 200.0)
                        .with_content(SceneContent::File("player.scn".into())),
                )
                .with_child(
                    "boss",
                    SceneNode::default()
                        .with_position(500.0, 500.0)
                        .with_content(SceneContent::File("boss.scn".into())),
                ),
        ),
    );
    let level_d_diff = Change::data_difference(&level_b, &level_d, &Default::default()).unwrap();
    let package_d = Package::default()
        .with_base("player.scn", player_b.to_owned())
        .with_base("boss.scn", boss_d.to_owned())
        .with_change("level.scn", level_d_diff);

    let result_a_b = package_a.patch(&package_b);
    let expected = Package::default()
        .with_base("level.scn", level_b)
        .with_base("player.scn", player_b.to_owned());
    assert_eq!(result_a_b, expected);

    let result_a_b_c = result_a_b.patch(&package_c);
    let expected = Package::default()
        .with_base("level.scn", level_c)
        .with_base("player.scn", player_b.to_owned())
        .with_base("enemy.scn", enemy_c.to_owned());
    assert_eq!(result_a_b_c, expected);

    let result_a_b_d = result_a_b.patch(&package_d);
    let expected = Package::default()
        .with_base("level.scn", level_d.to_owned())
        .with_base("player.scn", player_b.to_owned())
        .with_base("boss.scn", boss_d.to_owned());
    assert_eq!(result_a_b_d, expected);

    let result_a_b_c_d = result_a_b.patch(&package_c).patch(&package_d);
    let expected_level_c_d = Asset::Scene(
        SceneNode::default().with_content(
            SceneContent::default()
                .with_child(
                    "background",
                    SceneNode::default().with_primitive(Primitive::Sprite {
                        file: "winter.png".into(),
                        size: (1024.0, 1024.0),
                        pivot: Default::default(),
                    }),
                )
                .with_child(
                    "player",
                    SceneNode::default()
                        .with_position(100.0, 200.0)
                        .with_content(SceneContent::File("player.scn".into())),
                )
                .with_child(
                    "enemy",
                    SceneNode::default()
                        .with_position(900.0, 800.0)
                        .with_content(SceneContent::File("enemy.scn".into())),
                )
                .with_child(
                    "boss",
                    SceneNode::default()
                        .with_position(500.0, 500.0)
                        .with_content(SceneContent::File("boss.scn".into())),
                ),
        ),
    );
    let expected = Package::default()
        .with_base("level.scn", expected_level_c_d)
        .with_base("player.scn", player_b)
        .with_base("enemy.scn", enemy_c)
        .with_base("boss.scn", boss_d);
    assert_eq!(result_a_b_c_d, expected);
}

#[test]
fn test_editor_communication() {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
    enum ComponentType {
        Health,
        Attack,
        Position,
    }

    trait Component {
        fn get_type() -> ComponentType;
    }

    #[derive(
        Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize, ReflectIntermediate,
    )]
    struct Health(f32);

    impl Component for Health {
        fn get_type() -> ComponentType {
            ComponentType::Health
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ReflectIntermediate)]
    enum Attack {
        Direct(f32),
        Ranged { range: f32, value: f32 },
    }

    impl Component for Attack {
        fn get_type() -> ComponentType {
            ComponentType::Attack
        }
    }

    #[derive(
        Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize, ReflectIntermediate,
    )]
    struct Position {
        x: f32,
        y: f32,
    }

    impl Component for Position {
        fn get_type() -> ComponentType {
            ComponentType::Position
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
    struct Entity(usize);

    #[derive(Debug, Default)]
    struct World {
        health_components: HashMap<Entity, Health>,
        attack_components: HashMap<Entity, Attack>,
        position_components: HashMap<Entity, Position>,
    }

    impl World {
        fn add_health(&mut self, entity: Entity, health: Health) {
            self.health_components.insert(entity, health);
        }

        fn add_attack(&mut self, entity: Entity, attack: Attack) {
            self.attack_components.insert(entity, attack);
        }

        fn add_position(&mut self, entity: Entity, position: Position) {
            self.position_components.insert(entity, position);
        }

        fn health(&self, entity: Entity) -> Option<&Health> {
            self.health_components.get(&entity)
        }

        fn attack(&self, entity: Entity) -> Option<&Attack> {
            self.attack_components.get(&entity)
        }

        fn position(&self, entity: Entity) -> Option<&Position> {
            self.position_components.get(&entity)
        }

        fn entity_snapshot(&self, entity: Entity) -> Option<EntitySnapshot> {
            let mut components = HashMap::<_, _>::default();
            let mut exists = false;
            if let Some(health) = self.health_components.get(&entity) {
                components.insert(
                    ComponentType::Health,
                    crate::to_intermediate(health).unwrap(),
                );
                exists = true;
            }
            if let Some(attack) = self.attack_components.get(&entity) {
                components.insert(
                    ComponentType::Attack,
                    crate::to_intermediate(attack).unwrap(),
                );
                exists = true;
            }
            if let Some(position) = self.position_components.get(&entity) {
                components.insert(
                    ComponentType::Position,
                    crate::to_intermediate(position).unwrap(),
                );
                exists = true;
            }
            if exists {
                Some(EntitySnapshot { entity, components })
            } else {
                None
            }
        }

        fn apply_editor_change(&mut self, change: &EditorChange) {
            match change.component {
                ComponentType::Health => {
                    if let Some(data) = self.health_components.get_mut(&change.entity) {
                        data.patch_change(&change.change);
                    }
                }
                ComponentType::Attack => {
                    if let Some(data) = self.attack_components.get_mut(&change.entity) {
                        data.patch_change(&change.change);
                    }
                }
                ComponentType::Position => {
                    if let Some(data) = self.position_components.get_mut(&change.entity) {
                        data.patch_change(&change.change);
                    }
                }
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct EntitySnapshot {
        entity: Entity,
        components: HashMap<ComponentType, Intermediate>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct EditorChange {
        entity: Entity,
        component: ComponentType,
        change: Change,
    }

    #[derive(Default)]
    struct Editor {
        selected: Option<EntitySnapshot>,
    }

    impl Editor {
        fn change_selected<T, F>(&mut self, mut f: F) -> Option<EditorChange>
        where
            T: Component + Serialize + DeserializeOwned + std::fmt::Debug,
            F: FnMut(T) -> T,
        {
            let type_ = T::get_type();
            let snapshot = self.selected.as_mut()?;
            let entity = snapshot.entity;
            let value = snapshot.components.get_mut(&type_)?;
            let data = crate::from_intermediate(value).unwrap();
            let data = f(data);
            let serialized = crate::to_intermediate(&data).unwrap();
            let change = Change::difference(value, &serialized, &Default::default());
            if change.is_same() {
                return None;
            }
            *value = serialized;
            Some(EditorChange {
                entity,
                component: type_,
                change,
            })
        }
    }

    let (game_sender, editor_receiver) = channel();
    let (editor_sender, game_receiver) = channel();

    let mut world = World::default();
    world.add_position(Entity(0), Default::default());
    world.add_health(Entity(0), Health(100.0));
    world.add_attack(Entity(0), Attack::Direct(0.0));

    let snapshot = world.entity_snapshot(Entity(0)).unwrap();
    let snapshot = pot::to_vec(&snapshot).unwrap();
    game_sender.send(snapshot).unwrap();

    let mut editor = Editor::default();
    let snapshot = editor_receiver.recv().unwrap();
    editor.selected = Some(pot::from_slice::<EntitySnapshot>(&snapshot).unwrap());
    let change = editor
        .change_selected::<Position, _>(|position| Position {
            x: position.x + 10.0,
            y: position.y,
        })
        .unwrap();
    let change = pot::to_vec(&change).unwrap();
    editor_sender.send(change).unwrap();
    let change = editor
        .change_selected::<Health, _>(|health| Health(health.0 + 10.0))
        .unwrap();
    let change = pot::to_vec(&change).unwrap();
    editor_sender.send(change).unwrap();
    let change = editor
        .change_selected::<Attack, _>(|_| Attack::Ranged {
            range: 10.0,
            value: 10.0,
        })
        .unwrap();
    let change = pot::to_vec(&change).unwrap();
    editor_sender.send(change).unwrap();

    while let Ok(change) = game_receiver.try_recv() {
        let change = pot::from_slice::<EditorChange>(&change).unwrap();
        world.apply_editor_change(&change);
    }
    assert_eq!(
        world.position(Entity(0)).unwrap(),
        &Position { x: 10.0, y: 0.0 }
    );
    assert_eq!(world.health(Entity(0)).unwrap(), &Health(110.0));
    assert_eq!(
        world.attack(Entity(0)).unwrap(),
        &Attack::Ranged {
            range: 10.0,
            value: 10.0
        }
    );
}

#[test]
fn test_text_format() {
    macro_rules! lines {
        ($( $line:literal )+) => {
            vec![ $( $line ),+ ].join("\n")
        };
    }

    assert_eq!(crate::to_string_compact(&true).unwrap(), "true");
    assert_eq!(crate::to_string_pretty(&true).unwrap(), "true");

    assert_eq!(crate::to_string_compact(&42).unwrap(), "42_i32");
    assert_eq!(crate::to_string_pretty(&42).unwrap(), "42_i32");

    assert_eq!(crate::to_string_compact(&'@').unwrap(), "'@'");
    assert_eq!(crate::to_string_pretty(&'@').unwrap(), "'@'");

    assert_eq!(
        crate::to_string_compact("Hello World!").unwrap(),
        r#""Hello World!""#
    );
    assert_eq!(
        crate::to_string_pretty("Hello World!").unwrap(),
        r#""Hello World!""#
    );

    assert_eq!(
        crate::to_string_compact(&Intermediate::Bytes(b"Hello World!".to_vec())).unwrap(),
        "0x48656c6c6f20576f726c6421"
    );
    assert_eq!(
        crate::to_string_pretty(&Intermediate::Bytes(b"Hello World!".to_vec())).unwrap(),
        "0x48656c6c6f20576f726c6421"
    );

    assert_eq!(crate::to_string_compact(&Option::<()>::None).unwrap(), "?");
    assert_eq!(crate::to_string_pretty(&Option::<()>::None).unwrap(), "?");

    assert_eq!(crate::to_string_compact(&Some(42)).unwrap(), "?=42_i32");
    assert_eq!(crate::to_string_pretty(&Some(42)).unwrap(), "? = 42_i32");

    assert_eq!(crate::to_string_compact(&()).unwrap(), "!");
    assert_eq!(crate::to_string_pretty(&()).unwrap(), "!");

    assert_eq!(crate::to_string_compact(&UnitStruct).unwrap(), "#!");
    assert_eq!(crate::to_string_pretty(&UnitStruct).unwrap(), "#!");

    assert_eq!(crate::to_string_compact(&Enum::Unit).unwrap(), "@Unit!");
    assert_eq!(crate::to_string_pretty(&Enum::Unit).unwrap(), "@Unit !");

    assert_eq!(
        crate::to_string_compact(&NewTypeStruct(true)).unwrap(),
        "$=true"
    );
    assert_eq!(
        crate::to_string_pretty(&NewTypeStruct(true)).unwrap(),
        "$ = true"
    );

    assert_eq!(
        crate::to_string_compact(&Enum::NewType(UnitStruct)).unwrap(),
        "@NewType$=#!"
    );
    assert_eq!(
        crate::to_string_pretty(&Enum::NewType(UnitStruct)).unwrap(),
        "@NewType $ = #!"
    );

    assert_eq!(
        crate::to_string_compact(&vec![0, 1, 2]).unwrap(),
        "[0_i32,1_i32,2_i32]"
    );
    assert_eq!(
        crate::to_string_pretty(&vec![0, 1, 2]).unwrap(),
        lines! {
            "["
            "  0_i32,"
            "  1_i32,"
            "  2_i32"
            "]"
        }
    );
    assert_eq!(
        crate::to_string(
            &vec![0, 1, 2],
            TextConfig::default().with_style(TextConfigStyle::pretty(None))
        )
        .unwrap(),
        "[0_i32, 1_i32, 2_i32]"
    );

    assert_eq!(
        crate::to_string_compact(&(0, 1, 2)).unwrap(),
        "(0_i32,1_i32,2_i32)"
    );
    assert_eq!(
        crate::to_string_pretty(&(0, 1, 2)).unwrap(),
        lines! {
            "("
            "  0_i32,"
            "  1_i32,"
            "  2_i32"
            ")"
        }
    );
    assert_eq!(
        crate::to_string(
            &(0, 1, 2),
            TextConfig::default().with_style(TextConfigStyle::pretty(None))
        )
        .unwrap(),
        "(0_i32, 1_i32, 2_i32)"
    );

    assert_eq!(
        crate::to_string_compact(&TupleStruct(true, 42)).unwrap(),
        "#(true,42_u64)"
    );
    assert_eq!(
        crate::to_string_pretty(&TupleStruct(true, 42)).unwrap(),
        lines! {
            "# ("
            "  true,"
            "  42_u64"
            ")"
        }
    );
    assert_eq!(
        crate::to_string(
            &TupleStruct(true, 42),
            TextConfig::default().with_style(TextConfigStyle::pretty(None))
        )
        .unwrap(),
        "# (true, 42_u64)"
    );

    assert_eq!(
        crate::to_string_compact(&Enum::Tuple(true, 42)).unwrap(),
        "@Tuple(true,42_u64)"
    );
    assert_eq!(
        crate::to_string_pretty(&Enum::Tuple(true, 42)).unwrap(),
        lines! {
            "@Tuple ("
            "  true,"
            "  42_u64"
            ")"
        }
    );
    assert_eq!(
        crate::to_string(
            &Enum::Tuple(true, 42),
            TextConfig::default().with_style(TextConfigStyle::pretty(None))
        )
        .unwrap(),
        "@Tuple (true, 42_u64)"
    );

    try_until(
        100,
        || {
            crate::to_string_compact(
                &map! {"a".to_owned() => 0, "b".to_owned() => 1, "c".to_owned() => 2},
            )
            .unwrap()
        },
        r#"{"a":0_i32,"b":1_i32,"c":2_i32}"#.to_owned(),
    );
    try_until(
        100,
        || {
            crate::to_string_pretty(
                &map! {"a".to_owned() => 0, "b".to_owned() => 1, "c".to_owned() => 2},
            )
            .unwrap()
        },
        lines! {
            r#"{"#
            r#"  "a": 0_i32,"#
            r#"  "b": 1_i32,"#
            r#"  "c": 2_i32"#
            r#"}"#
        }
        .to_owned(),
    );
    try_until(
        100,
        || {
            crate::to_string(
                &map! {"a".to_owned() => 0, "b".to_owned() => 1, "c".to_owned() => 2},
                TextConfig::default().with_style(TextConfigStyle::pretty(None)),
            )
            .unwrap()
        },
        r#"{"a": 0_i32, "b": 1_i32, "c": 2_i32}"#.to_owned(),
    );

    try_until(200, || {
        crate::to_string_compact(&Struct {
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
        }).unwrap()
    }, r#"#{bool_value:true,i8_value:-1_i8,i16_value:2_i16,i32_value:-3_i32,i64_value:4_i64,i128_value:-5_i128,u8_value:6_u8,u16_value:7_u16,u32_value:8_u32,u64_value:9_u64,u128_value:10_u128,f32_value:1.1_f32,f64_value:1.2_f64,char_value:'@',string_value:"hello",tuple:(false,13_u64),bytes:[14_u8,15_u8,16_u8,17_u8,18_u8,19_u8],option:?=#!,list:[20_u64,21_u64,23_u64],set:[21_u64,23_u64,20_u64],string_map:{"b":25_u64,"a":24_u64},integer_map:{27_u64:28_u64,29_u64:30_u64},enum_value:@Struct#{scalar:3.1_f32,text:"world"},new_type_struct:$=true,tuple_struct:#(false,32_u64)}"#.to_owned());
    try_until(
        200,
        || {
            crate::to_string_pretty(&Struct {
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
            })
            .unwrap()
        },
        lines! {
            r#"# {"#
            r#"  bool_value: true,"#
            r#"  i8_value: -1_i8,"#
            r#"  i16_value: 2_i16,"#
            r#"  i32_value: -3_i32,"#
            r#"  i64_value: 4_i64,"#
            r#"  i128_value: -5_i128,"#
            r#"  u8_value: 6_u8,"#
            r#"  u16_value: 7_u16,"#
            r#"  u32_value: 8_u32,"#
            r#"  u64_value: 9_u64,"#
            r#"  u128_value: 10_u128,"#
            r#"  f32_value: 1.1_f32,"#
            r#"  f64_value: 1.2_f64,"#
            r#"  char_value: '@',"#
            r#"  string_value: "hello","#
            r#"  tuple: ("#
            r#"    false,"#
            r#"    13_u64"#
            r#"  ),"#
            r#"  bytes: ["#
            r#"    14_u8,"#
            r#"    15_u8,"#
            r#"    16_u8,"#
            r#"    17_u8,"#
            r#"    18_u8,"#
            r#"    19_u8"#
            r#"  ],"#
            r#"  option: ? = #!,"#
            r#"  list: ["#
            r#"    20_u64,"#
            r#"    21_u64,"#
            r#"    23_u64"#
            r#"  ],"#
            r#"  set: ["#
            r#"    21_u64,"#
            r#"    23_u64,"#
            r#"    20_u64"#
            r#"  ],"#
            r#"  string_map: {"#
            r#"    "b": 25_u64,"#
            r#"    "a": 24_u64"#
            r#"  },"#
            r#"  integer_map: {"#
            r#"    27_u64: 28_u64,"#
            r#"    29_u64: 30_u64"#
            r#"  },"#
            r#"  enum_value: @Struct # {"#
            r#"    scalar: 3.1_f32,"#
            r#"    text: "world""#
            r#"  },"#
            r#"  new_type_struct: $ = true,"#
            r#"  tuple_struct: # ("#
            r#"    false,"#
            r#"    32_u64"#
            r#"  )"#
            r#"}"#
        },
    );
    try_until(200, || {
        crate::to_string(&Struct {
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
        }, TextConfig::default().with_style(TextConfigStyle::pretty(None))).unwrap()
    }, r#"# {bool_value: true, i8_value: -1_i8, i16_value: 2_i16, i32_value: -3_i32, i64_value: 4_i64, i128_value: -5_i128, u8_value: 6_u8, u16_value: 7_u16, u32_value: 8_u32, u64_value: 9_u64, u128_value: 10_u128, f32_value: 1.1_f32, f64_value: 1.2_f64, char_value: '@', string_value: "hello", tuple: (false, 13_u64), bytes: [14_u8, 15_u8, 16_u8, 17_u8, 18_u8, 19_u8], option: ? = #!, list: [20_u64, 21_u64, 23_u64], set: [21_u64, 23_u64, 20_u64], string_map: {"b": 25_u64, "a": 24_u64}, integer_map: {27_u64: 28_u64, 29_u64: 30_u64}, enum_value: @Struct # {scalar: 3.1_f32, text: "world"}, new_type_struct: $ = true, tuple_struct: # (false, 32_u64)}"#.to_owned());

    assert_eq!(
        crate::to_string_compact(&Enum::Struct {
            scalar: 4.2,
            text: "Hello World!".to_owned()
        })
        .unwrap(),
        r#"@Struct#{scalar:4.2_f32,text:"Hello World!"}"#
    );
    assert_eq!(
        crate::to_string_pretty(&Enum::Struct {
            scalar: 4.2,
            text: "Hello World!".to_owned()
        })
        .unwrap(),
        lines! {
            r#"@Struct # {"#
            r#"  scalar: 4.2_f32,"#
            r#"  text: "Hello World!""#
            r#"}"#
        }
    );
    assert_eq!(
        crate::to_string(
            &Enum::Struct {
                scalar: 4.2,
                text: "Hello World!".to_owned()
            },
            TextConfig::default().with_style(TextConfigStyle::pretty(None))
        )
        .unwrap(),
        r#"@Struct # {scalar: 4.2_f32, text: "Hello World!"}"#
    );

    let content = lines! {
        r#"# {"#
        r#"  bool_value: true,"#
        r#"  i8_value: -1_i8,"#
        r#"  i16_value: 2_i16,"#
        r#"  i32_value: -3_i32,"#
        r#"  i64_value: 4_i64,"#
        r#"  i128_value: -5_i128,"#
        r#"  u8_value: 6_u8,"#
        r#"  u16_value: 7_u16,"#
        r#"  u32_value: 8_u32,"#
        r#"  u64_value: 9_u64,"#
        r#"  u128_value: 10_u128,"#
        r#"  f32_value: 1.1_f32,"#
        r#"  f64_value: 1.2_f64,"#
        r#"  char_value: '@',"#
        r#"  string_value: "hello","#
        r#"  tuple: ("#
        r#"    false,"#
        r#"    13_u64"#
        r#"  ),"#
        r#"  bytes: ["#
        r#"    14_u8,"#
        r#"    15_u8,"#
        r#"    16_u8,"#
        r#"    17_u8,"#
        r#"    18_u8,"#
        r#"    19_u8"#
        r#"  ],"#
        r#"  option: ? = #!,"#
        r#"  list: ["#
        r#"    20_u64,"#
        r#"    21_u64,"#
        r#"    23_u64"#
        r#"  ],"#
        r#"  set: ["#
        r#"    21_u64,"#
        r#"    23_u64,"#
        r#"    20_u64"#
        r#"  ],"#
        r#"  string_map: {"#
        r#"    "b": 25_u64,"#
        r#"    "a": 24_u64"#
        r#"  },"#
        r#"  integer_map: {"#
        r#"    27_u64: 28_u64,"#
        r#"    29_u64: 30_u64"#
        r#"  },"#
        r#"  enum_value: @Struct # {"#
        r#"    scalar: 3.1_f32,"#
        r#"    text: "world""#
        r#"  },"#
        r#"  new_type_struct: $ = true,"#
        r#"  tuple_struct: # ("#
        r#"    false,"#
        r#"    32_u64"#
        r#"  )"#
        r#"}"#
    };
    let provided = crate::from_str::<Struct>(&content).unwrap();
    let expected = Struct {
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
    assert_eq!(provided, expected);

    let content = r#"@Struct #{ scalar: 3.1_f32, text: "world" }"#;
    let provided = crate::intermediate_from_str(content).unwrap();
    let expected = crate::to_intermediate(&Enum::Struct {
        scalar: 3.1,
        text: "world".to_owned(),
    })
    .unwrap();
    assert_eq!(provided, expected);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Foo {
        a: i8,
        b: u8,
        c: f32,
    }

    let content = "#{a: -42, b: 42, c: 4.2}";
    let deserialized = crate::from_str::<Foo>(content).unwrap();
    assert_eq!(
        deserialized,
        Foo {
            a: -42,
            b: 42,
            c: 4.2
        }
    );
}

#[test]
fn test_schema() {
    use crate::schema::*;

    let mut provided = SchemaPackage::default().prefer_tree_id(true);
    UnitStruct::schema(&mut provided);
    let mut expected = SchemaPackage::default().prefer_tree_id(true);
    expected.with(
        SchemaIdTree::new::<UnitStruct>(),
        Schema::new(SchemaType::new_struct(SchemaTypeStruct::default()))
            .description("Unit struct."),
    );
    assert_eq!(provided, expected);

    let mut provided = SchemaPackage::default().prefer_tree_id(true);
    NewTypeStruct::schema(&mut provided);
    let mut expected = SchemaPackage::default().prefer_tree_id(true);
    expected.with(
        SchemaIdTree::new::<NewTypeStruct>(),
        Schema::new(SchemaType::new_tuple_struct(
            SchemaTypeTuple::default().item(
                SchemaTypeInstance::new(SchemaIdTree::new::<bool>()).description("Bool value."),
            ),
        ))
        .description("New type struct."),
    );
    assert_eq!(provided, expected);

    let mut provided = SchemaPackage::default().prefer_tree_id(true);
    TupleStruct::schema(&mut provided);
    let mut expected = SchemaPackage::default().prefer_tree_id(true);
    expected.with(
        SchemaIdTree::new::<TupleStruct>(),
        Schema::new(SchemaType::new_tuple_struct(
            SchemaTypeTuple::default()
                .item(SchemaIdTree::new::<bool>())
                .item(SchemaIdTree::new::<usize>()),
        ))
        .description("Tuple struct."),
    );
    assert_eq!(provided, expected);

    let mut expected = SchemaPackage::default().prefer_tree_id(true);
    expected.with(
        SchemaIdTree::new::<UnitStruct>(),
        Schema::new(SchemaType::new_struct(SchemaTypeStruct::default()))
            .description("Unit struct."),
    );
    expected.with(
        SchemaIdTree::new::<Enum>(),
        Schema::new(SchemaType::new_enum(
            SchemaTypeEnum::default()
                .variant("Unit", SchemaTypeEnumVariant::Empty)
                .variant(
                    "NewType",
                    SchemaTypeEnumVariant::new_tuple(
                        SchemaTypeTuple::default().item(SchemaIdTree::new::<UnitStruct>()),
                    ),
                )
                .variant(
                    "Tuple",
                    SchemaTypeEnumVariant::new_tuple(
                        SchemaTypeTuple::default()
                            .item(SchemaIdTree::new::<bool>())
                            .item(SchemaIdTree::new::<usize>()),
                    ),
                )
                .variant(
                    "Struct",
                    SchemaTypeEnumVariant::new_struct(
                        SchemaTypeStruct::default()
                            .field("text", SchemaIdTree::new::<String>())
                            .field("scalar", SchemaIdTree::new::<f32>()),
                    ),
                ),
        ))
        .description("Enum."),
    );
    try_until(
        100,
        || {
            let mut provided = SchemaPackage::default().prefer_tree_id(true);
            Enum::schema(&mut provided);
            provided
        },
        expected,
    );

    let mut expected = SchemaPackage::default().prefer_tree_id(true);
    expected.with(
        SchemaIdTree::new::<UnitStruct>(),
        Schema::new(SchemaType::new_struct(SchemaTypeStruct::default()))
            .description("Unit struct."),
    );
    expected.with(
        SchemaIdTree::new::<Enum>(),
        Schema::new(SchemaType::new_enum(
            SchemaTypeEnum::default()
                .variant("Unit", SchemaTypeEnumVariant::Empty)
                .variant(
                    "NewType",
                    SchemaTypeEnumVariant::new_tuple(
                        SchemaTypeTuple::default().item(SchemaIdTree::new::<UnitStruct>()),
                    ),
                )
                .variant(
                    "Tuple",
                    SchemaTypeEnumVariant::new_tuple(
                        SchemaTypeTuple::default()
                            .item(SchemaIdTree::new::<bool>())
                            .item(SchemaIdTree::new::<usize>()),
                    ),
                )
                .variant(
                    "Struct",
                    SchemaTypeEnumVariant::new_struct(
                        SchemaTypeStruct::default()
                            .field("text", SchemaIdTree::new::<String>())
                            .field("scalar", SchemaIdTree::new::<f32>()),
                    ),
                ),
        ))
        .description("Enum."),
    );
    expected.with(
        SchemaIdTree::new::<NewTypeStruct>(),
        Schema::new(SchemaType::new_tuple_struct(
            SchemaTypeTuple::default().item(
                SchemaTypeInstance::new(SchemaIdTree::new::<bool>()).description("Bool value."),
            ),
        ))
        .description("New type struct."),
    );
    expected.with(
        SchemaIdTree::new::<TupleStruct>(),
        Schema::new(SchemaType::new_tuple_struct(
            SchemaTypeTuple::default()
                .item(SchemaIdTree::new::<bool>())
                .item(SchemaIdTree::new::<usize>()),
        ))
        .description("Tuple struct."),
    );
    expected.with(
        SchemaIdTree::new::<Struct>(),
        Schema::new(SchemaType::new_struct(
            SchemaTypeStruct::default()
                .field("bool_value", SchemaIdTree::new::<bool>())
                .field("i8_value", SchemaIdTree::new::<i8>())
                .field("i16_value", SchemaIdTree::new::<i16>())
                .field("i32_value", SchemaIdTree::new::<i32>())
                .field("i64_value", SchemaIdTree::new::<i64>())
                .field("i128_value", SchemaIdTree::new::<i128>())
                .field("u8_value", SchemaIdTree::new::<u8>())
                .field("u16_value", SchemaIdTree::new::<u16>())
                .field("u32_value", SchemaIdTree::new::<u32>())
                .field("u64_value", SchemaIdTree::new::<u64>())
                .field("u128_value", SchemaIdTree::new::<u128>())
                .field("f32_value", SchemaIdTree::new::<f32>())
                .field("f64_value", SchemaIdTree::new::<f64>())
                .field("char_value", SchemaIdTree::new::<char>())
                .field("string_value", SchemaIdTree::new::<String>())
                .field("tuple", SchemaIdTree::new::<(bool, usize)>())
                .field("bytes", SchemaIdTree::new::<Vec<u8>>())
                .field("option", SchemaIdTree::new::<Option<UnitStruct>>())
                .field("list", SchemaIdTree::new::<Vec<usize>>())
                .field("set", SchemaIdTree::new::<HashSet<usize>>())
                .field("string_map", SchemaIdTree::new::<HashMap<String, usize>>())
                .field("integer_map", SchemaIdTree::new::<HashMap<usize, usize>>())
                .field("enum_value", SchemaIdTree::new::<Enum>())
                .field("new_type_struct", SchemaIdTree::new::<NewTypeStruct>())
                .field("tuple_struct", SchemaIdTree::new::<TupleStruct>()),
        ))
        .description("Struct."),
    );
    try_until(
        100,
        || {
            let mut provided = SchemaPackage::default().prefer_tree_id(true);
            Struct::schema(&mut provided);
            provided
        },
        expected,
    );
}
