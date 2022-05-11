use crate::{from_intermediate, Change, Intermediate};
use serde::de::DeserializeOwned;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

pub trait ReflectIntermediate {
    fn patch_change(&mut self, _change: &Change) {}

    fn before_patch_change(&mut self) {}

    fn after_patch_change(&mut self) {}
}

macro_rules! impl_reflect {
    (@atom $type:ty => $( $variant:ident ),+ ) => {
        impl ReflectIntermediate for $type {
            fn patch_change(&mut self, change: &Change) {
                #[allow(clippy::collapsible_match)]
                if let Change::Changed(v) = change {
                    match v {
                        $(
                            Intermediate::$variant(v) => if let Ok(v) = Self::try_from(*v) {
                                *self = v;
                            }
                        )+
                        _ => {}
                    }
                }
            }
        }
    };
}

impl ReflectIntermediate for () {}

impl_reflect! { @atom bool => Bool }
impl_reflect! { @atom i8 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 }
impl_reflect! { @atom i16 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 }
impl_reflect! { @atom i32 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 }
impl_reflect! { @atom i64 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 }
impl_reflect! { @atom i128 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 }
impl_reflect! { @atom isize => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 }
impl_reflect! { @atom u8 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char }
impl_reflect! { @atom u16 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 }
impl_reflect! { @atom u32 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char }
impl_reflect! { @atom u64 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char }
impl_reflect! { @atom u128 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char }
impl_reflect! { @atom usize => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 }
impl_reflect! { @atom f32 => I8, I16, U8, U16, F32 }
impl_reflect! { @atom f64 => I8, I16, I32, U8, U16, U32, F32, F64 }
impl_reflect! { @atom char => U8, U32, Char }

impl ReflectIntermediate for String {
    fn patch_change(&mut self, change: &Change) {
        if let Change::Changed(v) = change {
            match v {
                Intermediate::Char(v) => {
                    if let Ok(v) = Self::try_from(*v) {
                        *self = v;
                    }
                }
                Intermediate::String(v) => *self = v.to_owned(),
                _ => {}
            }
        }
    }
}

impl<T> ReflectIntermediate for Option<T>
where
    T: ReflectIntermediate + DeserializeOwned,
{
    fn patch_change(&mut self, change: &Change) {
        match change {
            Change::Changed(v) => {
                if let Ok(v) = from_intermediate(v) {
                    *self = v;
                }
            }
            Change::PartialChange(change) => {
                if let Some(content) = self {
                    content.patch_change(change);
                }
            }
            _ => {}
        }
    }
}

impl<T, E> ReflectIntermediate for Result<T, E>
where
    T: ReflectIntermediate + DeserializeOwned,
    E: ReflectIntermediate + DeserializeOwned,
{
    fn patch_change(&mut self, change: &Change) {
        match change {
            Change::Changed(v) => {
                if let Ok(v) = from_intermediate(v) {
                    *self = v;
                }
            }
            Change::PartialChange(change) => match self {
                Ok(content) => content.patch_change(change),
                Err(content) => content.patch_change(change),
            },
            _ => {}
        }
    }
}

impl<T, const N: usize> ReflectIntermediate for [T; N]
where
    T: ReflectIntermediate + DeserializeOwned,
{
    fn patch_change(&mut self, change: &Change) {
        match change {
            Change::Changed(Intermediate::Seq(v)) => {
                for (item, v) in self.iter_mut().zip(v.iter()) {
                    if let Ok(v) = from_intermediate(v) {
                        *item = v;
                    }
                }
            }
            Change::PartialSeq(v) => {
                for (index, change) in v {
                    if *index < N {
                        self[*index].patch_change(change);
                    }
                }
            }
            _ => {}
        }
    }
}

impl<T> ReflectIntermediate for (T,)
where
    T: ReflectIntermediate + DeserializeOwned,
{
    fn patch_change(&mut self, change: &Change) {
        match change {
            Change::Changed(Intermediate::Seq(v)) => {
                if let Some(v) = v.get(0) {
                    if let Ok(v) = from_intermediate(v) {
                        self.0 = v;
                    }
                }
            }
            Change::PartialSeq(v) => {
                for (index, change) in v {
                    if *index == 0 {
                        self.0.patch_change(change);
                    }
                }
            }
            _ => {}
        }
    }
}

macro_rules! impl_tuple {
    ( $( $id:ident : $index:tt ),+ ) => {
        impl< $( $id ),+ > ReflectIntermediate for ( $( $id ),+ )
        where
            $( $id: ReflectIntermediate + DeserializeOwned ),+
        {
            fn patch_change(&mut self, change: &Change) {
                match change {
                    Change::Changed(Intermediate::Seq(v)) => {
                        $(
                            if let Some(v) = v.get($index) {
                                if let Ok(v) = from_intermediate(v) {
                                    self.$index = v;
                                }
                            }
                        )+
                    }
                    Change::PartialSeq(v) => {
                        $(
                            if let Some((_,change)) = v.iter().find(|(i,_)| *i == $index) {
                                self.$index.patch_change(change);
                            }
                        )+
                    }
                    _ => {}
                }
            }
        }
    };
}

impl_tuple! { A:0, B:1 }
impl_tuple! { A:0, B:1, C:2 }
impl_tuple! { A:0, B:1, C:2, D:3 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17, S:18 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17, S:18, T:19 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17, S:18, T:19, U:20 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17, S:18, T:19, U:20, V:21 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17, S:18, T:19, U:20, V:21, X:22 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17, S:18, T:19, U:20, V:21, X:22, Y:23 }
impl_tuple! { A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11, M:12, N:13, O:14, P:15, Q:16, R:17, S:18, T:19, U:20, V:21, X:22, Y:23, Z:24 }

impl<T> ReflectIntermediate for Vec<T>
where
    T: ReflectIntermediate + DeserializeOwned,
{
    fn patch_change(&mut self, change: &Change) {
        match change {
            Change::Changed(v) => {
                if let Ok(v) = from_intermediate(v) {
                    *self = v;
                }
            }
            Change::PartialSeq(v) => {
                for (index, change) in v {
                    match change {
                        Change::Removed => {
                            self.remove(*index);
                        }
                        Change::Added(v) => {
                            if let Ok(v) = from_intermediate(v) {
                                self.insert(*index, v);
                            }
                        }
                        change => {
                            if let Some(item) = self.get_mut(*index) {
                                item.patch_change(change);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

impl<T> ReflectIntermediate for HashSet<T>
where
    T: ReflectIntermediate + DeserializeOwned + Hash + Eq + Clone,
{
    fn patch_change(&mut self, change: &Change) {
        match change {
            Change::Changed(v) => {
                if let Ok(v) = from_intermediate(v) {
                    *self = v;
                }
            }
            Change::PartialSeq(v) => {
                let mut data = self.iter().cloned().collect::<Vec<_>>();
                for (index, change) in v {
                    match change {
                        Change::Removed => {
                            data.remove(*index);
                        }
                        Change::Added(v) => {
                            if let Ok(v) = from_intermediate(v) {
                                data.insert(*index, v);
                            }
                        }
                        change => {
                            if let Some(item) = data.get_mut(*index) {
                                item.patch_change(change);
                            }
                        }
                    }
                }
                *self = data.into_iter().collect();
            }
            _ => {}
        }
    }
}

impl<K, V> ReflectIntermediate for HashMap<K, V>
where
    K: ReflectIntermediate + DeserializeOwned + Hash + Eq,
    V: ReflectIntermediate + DeserializeOwned,
{
    fn patch_change(&mut self, change: &Change) {
        match change {
            Change::Changed(v) => {
                if let Ok(v) = from_intermediate(v) {
                    *self = v;
                }
            }
            Change::PartialMap(v) => {
                for (key, change) in v {
                    if let Ok(key) = from_intermediate(key) {
                        match change {
                            Change::Removed => {
                                self.remove(&key);
                            }
                            Change::Added(v) => {
                                if let Ok(v) = from_intermediate(v) {
                                    self.insert(key, v);
                                }
                            }
                            change => {
                                if let Some(item) = self.get_mut(&key) {
                                    item.patch_change(change);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

impl<T> ReflectIntermediate for Box<T>
where
    T: ReflectIntermediate + DeserializeOwned,
{
    fn patch_change(&mut self, change: &Change) {
        match change {
            Change::Changed(v) => {
                if let Ok(v) = from_intermediate(v) {
                    *self = v;
                }
            }
            Change::PartialChange(change) => {
                self.patch_change(change);
            }
            _ => {}
        }
    }
}
