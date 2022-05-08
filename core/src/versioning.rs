use crate::{error::*, value::intermediate::Intermediate};
use petgraph::{algo::astar, Graph};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Change {
    Same,
    Removed,
    /// (value)
    Changed(Intermediate),
    /// (value)
    Added(Intermediate),
    /// (change)
    PartialChange(Box<Change>),
    /// (changes: [(index, change)])
    PartialSeq(Vec<(usize, Change)>),
    /// (changes: [(key, change)])
    PartialMap(Vec<(Intermediate, Change)>),
    /// (changes: [(name, change)])
    PartialStruct(Vec<(String, Change)>),
}

impl Change {
    pub fn partial_seq() -> Self {
        Change::PartialSeq(vec![])
    }

    pub fn partial_map() -> Self {
        Change::PartialMap(vec![])
    }

    pub fn partial_struct() -> Self {
        Change::PartialStruct(vec![])
    }

    pub fn partial_seq_item(mut self, index: usize, change: Change) -> Self {
        if let Self::PartialSeq(v) = &mut self {
            v.push((index, change));
        }
        self
    }

    pub fn partial_map_item(mut self, key: Intermediate, change: Change) -> Self {
        if let Self::PartialMap(v) = &mut self {
            if let Some(item) = v.iter_mut().find(|(k, _)| k == &key) {
                item.1 = change;
            } else {
                v.push((key, change));
            }
        }
        self
    }

    pub fn partial_struct_item(mut self, name: String, change: Change) -> Self {
        if let Self::PartialStruct(v) = &mut self {
            if let Some(item) = v.iter_mut().find(|(n, _)| n == &name) {
                item.1 = change;
            } else {
                v.push((name, change));
            }
        }
        self
    }

    pub fn is_same(&self) -> bool {
        matches!(self, Self::Same)
    }
}

pub fn difference(prev: &Intermediate, next: &Intermediate) -> Change {
    if prev == next {
        Change::Same
    } else {
        match (prev, next) {
            (Intermediate::Option(Some(prev)), Intermediate::Option(Some(next))) => {
                Change::PartialChange(Box::new(difference(prev, next)))
            }
            (Intermediate::NewTypeStruct(prev), Intermediate::NewTypeStruct(next)) => {
                Change::PartialChange(Box::new(difference(prev, next)))
            }
            (
                Intermediate::NewTypeVariant(prev_name, prev_index, prev_value),
                Intermediate::NewTypeVariant(next_name, next_index, next_value),
            ) => {
                if prev_name != next_name || prev_index != next_index {
                    Change::Changed(*next_value.to_owned())
                } else {
                    Change::PartialChange(Box::new(difference(prev_value, next_value)))
                }
            }
            (Intermediate::Seq(prev), Intermediate::Seq(next))
            | (Intermediate::Tuple(prev), Intermediate::Tuple(next))
            | (Intermediate::TupleStruct(prev), Intermediate::TupleStruct(next)) => {
                Change::PartialSeq(sequence_difference(prev, next))
            }
            (Intermediate::Map(prev), Intermediate::Map(next)) => {
                let mut result = vec![];
                for (nk, nv) in next {
                    if !prev.iter().any(|(pk, _)| pk == nk) {
                        result.push((nk.to_owned(), Change::Added(nv.to_owned())));
                    }
                }
                for (pk, _) in prev {
                    if !next.iter().any(|(nk, _)| pk == nk) {
                        result.push((pk.to_owned(), Change::Removed));
                    }
                }
                for (pk, pv) in prev {
                    if let Some((_, nv)) = next
                        .iter()
                        .find(|(nk, _)| pk == nk)
                        .filter(|(_, nv)| pv != nv)
                    {
                        let diff = difference(pv, nv);
                        if !diff.is_same() {
                            result.push((pk.to_owned(), Change::PartialChange(Box::new(diff))));
                        }
                    }
                }
                Change::PartialMap(result)
            }
            (Intermediate::Struct(prev), Intermediate::Struct(next))
            | (Intermediate::StructVariant(_, _, prev), Intermediate::StructVariant(_, _, next)) => {
                let mut result = vec![];
                for (nk, nv) in next {
                    if !prev.iter().any(|(pk, _)| pk == nk) {
                        result.push((nk.to_owned(), Change::Added(nv.to_owned())));
                    }
                }
                for (pk, _) in prev {
                    if !next.iter().any(|(nk, _)| pk == nk) {
                        result.push((pk.to_owned(), Change::Removed));
                    }
                }
                for (pk, pv) in prev {
                    if let Some((_, nv)) = next
                        .iter()
                        .find(|(nk, _)| pk == nk)
                        .filter(|(_, nv)| pv != nv)
                    {
                        let diff = difference(pv, nv);
                        if !diff.is_same() {
                            result.push((pk.to_owned(), Change::PartialChange(Box::new(diff))));
                        }
                    }
                }
                Change::PartialStruct(result)
            }
            _ => Change::Changed(next.to_owned()),
        }
    }
}

pub fn sequence_difference(prev: &[Intermediate], next: &[Intermediate]) -> Vec<(usize, Change)> {
    if prev.is_empty() && next.is_empty() {
        return vec![];
    } else if prev.is_empty() {
        return next
            .iter()
            .enumerate()
            .map(|(i, v)| (i, Change::Added(v.to_owned())))
            .collect();
    } else if next.is_empty() {
        return (0..prev.len()).map(|_| (0, Change::Removed)).collect();
    }

    /// (prev index, next index)
    type Location = (usize, usize);

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    enum Diff {
        Unchanged,
        Changed,
        Added,
        Removed,
    }

    impl Diff {
        fn cost(&self) -> usize {
            match self {
                Self::Added => 8,
                Self::Removed => 9,
                Self::Unchanged => 10,
                Self::Changed => 11,
            }
        }
    }

    let cols = prev.len() + 1;
    let rows = next.len() + 1;
    let mut graph = Graph::<Location, Diff>::with_capacity(
        cols * rows,
        (cols - 1) * rows + (rows - 1) * cols + (cols - 1) * (rows - 1),
    );
    let mut nodes = Vec::with_capacity(cols * rows);
    for row in 0..rows {
        for col in 0..cols {
            nodes.push(graph.add_node((col, row)));
        }
    }
    let get_node = |col, row| nodes[row * cols + col];
    for row in 0..rows {
        for col in 0..(cols - 1) {
            graph.add_edge(get_node(col, row), get_node(col + 1, row), Diff::Removed);
        }
    }
    for col in 0..cols {
        for row in 0..(rows - 1) {
            graph.add_edge(get_node(col, row), get_node(col, row + 1), Diff::Added);
        }
    }
    for (col, prev) in prev.iter().enumerate().take(cols - 1) {
        for (row, next) in next.iter().enumerate().take(rows - 1) {
            if prev == next {
                graph.add_edge(
                    get_node(col, row),
                    get_node(col + 1, row + 1),
                    Diff::Unchanged,
                );
            } else {
                graph.add_edge(
                    get_node(col, row),
                    get_node(col + 1, row + 1),
                    Diff::Changed,
                );
            }
        }
    }
    let finish = *nodes.last().unwrap();
    astar(
        &graph,
        *nodes.first().unwrap(),
        |n| n == finish,
        |e| e.weight().cost(),
        |_| 0,
    )
    .map(|(_, path)| {
        let mut pos = 0;
        path.windows(2)
            .filter_map(|chunk| {
                let diff = graph
                    .find_edge(chunk[0], chunk[1])
                    .and_then(|e| graph.edge_weight(e))?;
                let old_pos = pos;
                match diff {
                    Diff::Unchanged => {
                        pos += 1;
                        None
                    }
                    Diff::Changed => {
                        pos += 1;
                        Some((old_pos, Change::Changed(next[old_pos].to_owned())))
                    }
                    Diff::Removed => Some((old_pos, Change::Removed)),
                    Diff::Added => {
                        pos += 1;
                        Some((old_pos, Change::Added(next[old_pos].to_owned())))
                    }
                }
            })
            .collect()
    })
    .unwrap_or_default()
}

pub fn patch(value: &Intermediate, change: &Change) -> Result<Option<Intermediate>> {
    match change {
        Change::Same => Ok(Some(value.to_owned())),
        Change::Removed => Ok(None),
        Change::Changed(v) => Ok(Some(v.to_owned())),
        Change::Added(_) => Err(Error::CannotAdd(value.to_owned())),
        Change::PartialChange(change) => patch(value, change),
        Change::PartialSeq(changes) => {
            fn implement(
                v: &[Intermediate],
                changes: &[(usize, Change)],
            ) -> Result<Vec<Intermediate>> {
                let mut result = v.to_owned();
                for (index, change) in changes {
                    match change {
                        Change::Removed => {
                            result.remove(*index);
                        }
                        Change::Changed(v) => {
                            if let Some(item) = result.get_mut(*index) {
                                *item = v.to_owned();
                            }
                        }
                        Change::Added(v) => result.insert(*index, v.to_owned()),
                        change => {
                            if let Some(item) = result.get_mut(*index) {
                                if let Some(patched) = patch(item, change)? {
                                    *item = patched;
                                }
                            }
                        }
                    }
                }
                Ok(result)
            }

            match value {
                Intermediate::Seq(v) => Ok(Some(Intermediate::Seq(implement(v, changes)?))),
                Intermediate::Tuple(v) => Ok(Some(Intermediate::Tuple(implement(v, changes)?))),
                Intermediate::TupleStruct(v) => {
                    Ok(Some(Intermediate::TupleStruct(implement(v, changes)?)))
                }
                _ => Err(Error::NotSeq(value.to_owned())),
            }
        }
        Change::PartialMap(changes) => match value {
            Intermediate::Map(v) => {
                let mut result = v.to_owned();
                for (key, change) in changes {
                    match change {
                        Change::Removed => {
                            if let Some(index) = result.iter().position(|(k, _)| k == key) {
                                result.remove(index);
                            }
                        }
                        Change::Changed(v) => {
                            if let Some(index) = result.iter().position(|(k, _)| k == key) {
                                if let Some(item) = result.get_mut(index) {
                                    item.1 = v.to_owned();
                                }
                            }
                        }
                        Change::Added(v) => {
                            if let Some(item) = result.iter_mut().find(|(k, _)| k == key) {
                                item.1 = v.to_owned();
                            } else {
                                result.push((key.to_owned(), v.to_owned()))
                            }
                        }
                        change => {
                            if let Some(item) = result.iter_mut().find(|(k, _)| k == key) {
                                if let Some(patched) = patch(&item.1, change)? {
                                    item.1 = patched;
                                }
                            }
                        }
                    }
                }
                Ok(Some(Intermediate::Map(result)))
            }
            _ => Err(Error::NotMap(value.to_owned())),
        },
        Change::PartialStruct(changes) => {
            fn implement(
                v: &[(String, Intermediate)],
                changes: &[(String, Change)],
            ) -> Result<Vec<(String, Intermediate)>> {
                let mut result = v.to_owned();
                for (key, change) in changes {
                    match change {
                        Change::Removed => {
                            if let Some(index) = result.iter().position(|(k, _)| k == key) {
                                result.remove(index);
                            }
                        }
                        Change::Changed(v) => {
                            if let Some(index) = result.iter().position(|(k, _)| k == key) {
                                if let Some(item) = result.get_mut(index) {
                                    item.1 = v.to_owned();
                                }
                            }
                        }
                        Change::Added(v) => {
                            if let Some(item) = result.iter_mut().find(|(k, _)| k == key) {
                                item.1 = v.to_owned();
                            } else {
                                result.push((key.to_owned(), v.to_owned()))
                            }
                        }
                        change => {
                            if let Some(item) = result.iter_mut().find(|(k, _)| k == key) {
                                if let Some(patched) = patch(&item.1, change)? {
                                    item.1 = patched;
                                }
                            }
                        }
                    }
                }
                Ok(result)
            }

            match value {
                Intermediate::Struct(v) => Ok(Some(Intermediate::Struct(implement(v, changes)?))),
                Intermediate::StructVariant(n, i, v) => Ok(Some(Intermediate::StructVariant(
                    n.to_owned(),
                    *i,
                    implement(v, changes)?,
                ))),
                _ => Err(Error::NotMap(value.to_owned())),
            }
        }
    }
}

pub fn data_difference<P, N>(prev: &P, next: &N) -> Result<Change>
where
    P: Serialize,
    N: Serialize,
{
    let prev = crate::to_intermediate(prev)?;
    let next = crate::to_intermediate(next)?;
    Ok(difference(&prev, &next))
}

pub fn data_patch<T>(data: &T, change: &Change) -> Result<Option<T>>
where
    T: Serialize + DeserializeOwned,
{
    let serialized = crate::to_intermediate(data)?;
    let patched = match patch(&serialized, change)? {
        Some(patched) => patched,
        None => return Ok(None),
    };
    Ok(Some(crate::from_intermediate::<T>(&patched)?))
}
