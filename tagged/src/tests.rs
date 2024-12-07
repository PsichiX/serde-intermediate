use crate::*;

#[test]
fn test_remote() {
    TaggedIntermediate::register::<bool>();
    TaggedIntermediate::register_named::<String>("String");

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct FooDef {
        a: TaggedIntermediate,
        b: TaggedIntermediate,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(into = "FooDef")]
    #[serde(from = "FooDef")]
    struct Foo {
        a: bool,
        b: String,
    }

    impl From<FooDef> for Foo {
        fn from(value: FooDef) -> Self {
            Self {
                a: value.a.decode().unwrap(),
                b: value.b.decode().unwrap(),
            }
        }
    }

    impl From<Foo> for FooDef {
        fn from(val: Foo) -> Self {
            FooDef {
                a: TaggedIntermediate::encode(&val.a).unwrap(),
                b: TaggedIntermediate::encode(&val.b).unwrap(),
            }
        }
    }

    let data = Foo {
        a: true,
        b: "Hello World!".to_owned(),
    };
    let serialized = serde_json::to_string(&data).unwrap();
    let deserialized = serde_json::from_str::<Foo>(&serialized).unwrap();
    assert_eq!(data, deserialized);

    TaggedIntermediate::unregister::<bool>();
    TaggedIntermediate::unregister::<String>();
}

#[test]
fn test_container() {
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct Foo {
        a: TaggedIntermediate,
        b: TaggedIntermediate,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct Bar {
        c: f32,
        d: usize,
    }

    TaggedIntermediate::register_named::<Bar>("Bar");
    TaggedIntermediate::register_named::<String>("String");

    let a = Bar { c: 1.2, d: 3 };
    let b = "hello world!".to_owned();
    let data = Foo {
        a: TaggedIntermediate::encode(&a).unwrap(),
        b: TaggedIntermediate::encode(&b).unwrap(),
    };
    let serialized = serde_json::to_string_pretty(&data).unwrap();
    let deserialized = serde_json::from_str::<Foo>(&serialized).unwrap();
    assert_eq!(deserialized.a.decode::<Bar>().unwrap(), a);
    assert_eq!(deserialized.b.decode::<String>().unwrap(), b);

    TaggedIntermediate::unregister_all();
}
