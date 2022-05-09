#![cfg(test)]

use super::*;
use crate as serde_reflect_intermediate;
use serde::{Deserialize, Serialize};
use serde_intermediate::to_intermediate;
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

fn patch<T>(mut prev: T, next: T)
where
    T: ReflectIntermediate + PartialEq + std::fmt::Debug + Serialize,
{
    let change = Change::difference(
        &to_intermediate(&prev).unwrap(),
        &to_intermediate(&next).unwrap(),
    );
    prev.patch_change(&change);
    assert_eq!(prev, next);
}

#[test]
fn test_general() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Foo {
        a: bool,
        b: usize,
    }

    impl ReflectIntermediate for Foo {
        fn patch_change(&mut self, change: &Change) {
            match change {
                Change::Changed(v) => {
                    if let Ok(v) = from_intermediate(v) {
                        *self = v;
                    }
                }
                Change::PartialStruct(v) => {
                    for (name, change) in v {
                        match name.as_str() {
                            "a" => self.a.patch_change(change),
                            "b" => self.b.patch_change(change),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum Bar {
        A,
        B(bool),
        C(bool, usize),
        D { a: bool, b: usize },
    }

    impl ReflectIntermediate for Bar {
        fn patch_change(&mut self, change: &Change) {
            match change {
                Change::Changed(v) => {
                    if let Ok(v) = from_intermediate(v) {
                        *self = v;
                    }
                }
                Change::PartialChange(change) => {
                    if let Self::B(content) = self {
                        content.patch_change(change);
                    }
                }
                Change::PartialStruct(v) => {
                    if let Self::D { a, b } = self {
                        for (name, change) in v {
                            match name.as_str() {
                                "a" => a.patch_change(change),
                                "b" => b.patch_change(change),
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Zee(bool, usize);

    impl ReflectIntermediate for Zee {
        fn patch_change(&mut self, change: &Change) {
            match change {
                Change::Changed(v) => {
                    if let Ok(v) = from_intermediate(v) {
                        *self = v;
                    }
                }
                Change::PartialSeq(v) => {
                    for (index, change) in v {
                        match *index {
                            0 => self.0.patch_change(change),
                            1 => self.1.patch_change(change),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    patch(false, true);
    patch('a', 'b');
    patch(1.2, 3.0);
    patch(4usize, 5usize);
    patch("hello".to_owned(), "@".to_owned());
    patch(None, Some(true));
    patch(Err(true), Ok(42usize));
    patch([1, 2, 3], [1, 0, 3]);
    patch((1, 2, 3), (1, 0, 3));
    patch(vec![1, 2, 3], vec![1, 0, 3]);
    patch(vec![1, 2, 3], vec![1, 3]);
    patch(vec![1, 2, 3], vec![1, 2, 3, 4]);
    patch(set![1, 2, 3], set![1, 0, 3]);
    patch(set![1, 2, 3], set![1, 3]);
    patch(set![1, 2, 3], set![1, 2, 3, 4]);
    patch(
        map! {'a' => 1, 'b' => 2, 'c' => 3},
        map! {'a' => 1, 'b' => 0, 'c' => 3},
    );
    patch(
        map! {'a' => 1, 'b' => 2, 'c' => 3},
        map! {'a' => 1, 'c' => 3},
    );
    patch(
        map! {'a' => 1, 'b' => 2, 'c' => 3},
        map! {'a' => 1, 'b' => 0, 'c' => 3, 'd' => 4},
    );
    patch(Box::new(false), Box::new(true));
    patch(Foo { a: false, b: 0 }, Foo { a: true, b: 0 });
    patch(Foo { a: false, b: 0 }, Foo { a: true, b: 42 });
    patch(Foo { a: false, b: 0 }, Foo { a: false, b: 42 });
    patch(Bar::A, Bar::B(false));
    patch(Bar::B(false), Bar::B(true));
    patch(Bar::B(true), Bar::C(true, 0));
    patch(Bar::C(true, 0), Bar::C(true, 42));
    patch(Bar::C(true, 42), Bar::D { a: true, b: 0 });
    patch(Bar::D { a: true, b: 0 }, Bar::D { a: true, b: 42 });
    patch(Bar::D { a: true, b: 42 }, Bar::A);
    patch(Zee(false, 0), Zee(true, 0));
    patch(Zee(false, 0), Zee(true, 42));
    patch(Zee(false, 0), Zee(false, 42));
}

#[cfg(feature = "derive")]
#[test]
fn test_derive() {
    use crate::ReflectIntermediate;

    #[derive(Debug, Serialize, Deserialize, ReflectIntermediate, PartialEq)]
    struct Foo {
        a: bool,
        b: usize,
    }

    #[derive(Debug, Serialize, Deserialize, ReflectIntermediate, PartialEq)]
    enum Bar {
        A,
        B(bool),
        C(bool, usize),
        D { a: bool, b: usize },
    }

    #[derive(Debug, Serialize, Deserialize, ReflectIntermediate, PartialEq)]
    struct Zee(bool, usize);

    #[derive(Debug, Serialize, Deserialize, ReflectIntermediate, PartialEq)]
    struct Unit;

    patch(Unit, Unit);
    patch(Foo { a: false, b: 0 }, Foo { a: true, b: 0 });
    patch(Foo { a: false, b: 0 }, Foo { a: true, b: 42 });
    patch(Foo { a: false, b: 0 }, Foo { a: false, b: 42 });
    patch(Bar::A, Bar::B(false));
    patch(Bar::B(false), Bar::B(true));
    patch(Bar::B(true), Bar::C(true, 0));
    patch(Bar::C(true, 0), Bar::C(true, 42));
    patch(Bar::C(true, 42), Bar::D { a: true, b: 0 });
    patch(Bar::D { a: true, b: 0 }, Bar::D { a: true, b: 42 });
    patch(Bar::D { a: true, b: 42 }, Bar::A);
    patch(Zee(false, 0), Zee(true, 0));
    patch(Zee(false, 0), Zee(true, 42));
    patch(Zee(false, 0), Zee(false, 42));
}
