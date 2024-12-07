use crate::{from_intermediate, Change, Intermediate};
use serde::de::DeserializeOwned;
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    hash::Hash,
    marker::PhantomData,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    },
    ops::{Range, RangeInclusive},
    path::PathBuf,
    sync::{
        atomic::{
            AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16,
            AtomicU32, AtomicU64, AtomicU8, AtomicUsize,
        },
        Mutex,
    },
};

/// Trait used to enable patching changes directly into data.
/// Prefer to implement using `ReflectIntermediate` derive macro.
///
/// # Example
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use serde_intermediate::{Change, ReflectIntermediate};
///
/// #[derive(Debug, PartialEq, Serialize, Deserialize, ReflectIntermediate)]
/// struct Foo(bool);
///
/// let mut a = Foo(false);
/// let b = Foo(true);
/// let change = Change::data_difference(&a, &b, &Default::default()).unwrap();
/// a.patch_change(&change);
/// assert_eq!(a, b);
/// ```
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
                            #[allow(irrefutable_let_patterns)]
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
    (@cast $type:ty => $cast:ty => $( $variant:ident ),+ ) => {
        impl ReflectIntermediate for $type {
            fn patch_change(&mut self, change: &Change) {
                #[allow(clippy::collapsible_match)]
                if let Change::Changed(v) = change {
                    match v {
                        $(
                            #[allow(irrefutable_let_patterns)]
                            Intermediate::$variant(v) => if let Ok(v) = <$cast>::try_from(*v) {
                                if let Ok(v) = Self::try_from(v) {
                                    *self = v.into();
                                }
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
impl<T> ReflectIntermediate for PhantomData<T> {}

impl_reflect!(@atom bool => Bool);
impl_reflect!(@atom i8 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@atom i16 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@atom i32 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@atom i64 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@atom i128 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@atom isize => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@atom u8 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char);
impl_reflect!(@atom u16 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@atom u32 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char);
impl_reflect!(@atom u64 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char);
impl_reflect!(@atom u128 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char);
impl_reflect!(@atom usize => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@atom f32 => I8, I16, U8, U16, F32);
impl_reflect!(@atom f64 => I8, I16, I32, U8, U16, U32, F32, F64);
impl_reflect!(@atom char => U8, U32, Char);
impl_reflect!(@cast AtomicBool => bool => Bool);
impl_reflect!(@cast AtomicI8 => i8 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast AtomicI16 => i16 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast AtomicI32 => i32 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast AtomicI64 => i64 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast AtomicIsize => isize => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast AtomicU8 => u8 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char);
impl_reflect!(@cast AtomicU16 => u16 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast AtomicU32 => u32 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char);
impl_reflect!(@cast AtomicU64 => u64 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char);
impl_reflect!(@cast AtomicUsize => usize => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast NonZeroI8 => i8 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast NonZeroI16 => i16 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast NonZeroI32 => i32 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast NonZeroI64 => i64 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast NonZeroI128 => i128 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast NonZeroIsize => isize => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast NonZeroU8 => u8 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char);
impl_reflect!(@cast NonZeroU16 => u16 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);
impl_reflect!(@cast NonZeroU32 => u32 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char);
impl_reflect!(@cast NonZeroU64 => u64 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char);
impl_reflect!(@cast NonZeroU128 => u128 => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128, Char);
impl_reflect!(@cast NonZeroUsize => usize => Bool, I8, I16, I32, I64, I128, U8, U16, U32, U64, U128);

impl ReflectIntermediate for String {
    fn patch_change(&mut self, change: &Change) {
        if let Change::Changed(v) = change {
            match v {
                Intermediate::Char(v) => *self = v.to_string(),
                Intermediate::String(v) => *self = v.to_owned(),
                _ => {}
            }
        }
    }
}

impl ReflectIntermediate for PathBuf {
    fn patch_change(&mut self, change: &Change) {
        if let Change::Changed(v) = change {
            match v {
                Intermediate::Char(v) => *self = v.to_string().into(),
                Intermediate::String(v) => *self = v.into(),
                _ => {}
            }
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
                if let Some(v) = v.first() {
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

impl<T> ReflectIntermediate for VecDeque<T>
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

impl<T> ReflectIntermediate for BTreeSet<T>
where
    T: ReflectIntermediate + DeserializeOwned + Ord + Clone,
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

impl<T> ReflectIntermediate for LinkedList<T>
where
    T: ReflectIntermediate + DeserializeOwned + Clone,
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

impl<T> ReflectIntermediate for BinaryHeap<T>
where
    T: ReflectIntermediate + DeserializeOwned + Ord + Clone,
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

impl<K, V> ReflectIntermediate for BTreeMap<K, V>
where
    K: ReflectIntermediate + DeserializeOwned + Ord,
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

impl<T> ReflectIntermediate for Cell<T>
where
    T: ReflectIntermediate + DeserializeOwned,
{
    fn patch_change(&mut self, change: &Change) {
        match change {
            Change::Changed(v) => {
                if let Ok(v) = from_intermediate(v) {
                    self.set(v);
                }
            }
            Change::PartialChange(change) => {
                self.patch_change(change);
            }
            _ => {}
        }
    }
}

impl<T> ReflectIntermediate for Mutex<T>
where
    T: ReflectIntermediate + DeserializeOwned,
{
    fn patch_change(&mut self, change: &Change) {
        match change {
            Change::Changed(v) => {
                if let Ok(v) = from_intermediate(v) {
                    if let Ok(ref mut mutex) = self.try_lock() {
                        **mutex = v;
                    }
                }
            }
            Change::PartialChange(change) => {
                self.patch_change(change);
            }
            _ => {}
        }
    }
}

impl<T> ReflectIntermediate for Range<T>
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
            Change::PartialStruct(v) => {
                for (key, change) in v {
                    match key.as_str() {
                        "start" => self.start.patch_change(change),
                        "end" => self.end.patch_change(change),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

impl<T> ReflectIntermediate for RangeInclusive<T>
where
    T: ReflectIntermediate + DeserializeOwned + Clone,
{
    fn patch_change(&mut self, change: &Change) {
        match change {
            Change::Changed(v) => {
                if let Ok(v) = from_intermediate(v) {
                    *self = v;
                }
            }
            Change::PartialStruct(v) => {
                let (mut start, mut end) = self.clone().into_inner();
                for (key, change) in v {
                    match key.as_str() {
                        "start" => start.patch_change(change),
                        "end" => end.patch_change(change),
                        _ => {}
                    }
                }
                *self = Self::new(start, end);
            }
            _ => {}
        }
    }
}
