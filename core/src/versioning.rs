use crate::{error::*, value::intermediate::Intermediate};
use petgraph::{algo::astar, Graph};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Optimization hint used in calculating change between two intermediate data.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum DiffOptimizationHint {
    /// Don't assume any optimization.
    #[default]
    Default,
    /// Put entire new object if change information size is greater than source (old) size.
    SizeSource,
    /// Put entire new object if change information size is greater than target (new) size.
    SizeTarget,
    /// Put entire new object if change information size is greater than bytesize threshold.
    SizeValue(
        /// Bytesize threshold.
        usize,
    ),
    /// Put entire new object if change information size is greater than percentage of source size.
    SizePercentage(
        /// Percentage threshold.
        f64,
    ),
}

/// Change calculation options.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct DiffOptions {
    /// Optimization hint.
    pub optimization_hint: DiffOptimizationHint,
}

impl DiffOptions {
    pub fn optimization_hint(mut self, hint: DiffOptimizationHint) -> Self {
        self.optimization_hint = hint;
        self
    }
}

/// Information about change between two intermediate data.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Change {
    /// Values are same.
    Same,
    /// Value was removed.
    Removed,
    /// Value was entirely changed.
    Changed(
        /// Value.
        Intermediate,
    ),
    /// Value was added.
    Added(
        /// Value.
        Intermediate,
    ),
    /// Value was partially changed.
    PartialChange(
        /// Content change.
        Box<Change>,
    ),
    /// Sequence of values was partially changed.
    PartialSeq(
        /// List of changes: `(index, change)`.
        Vec<(usize, Change)>,
    ),
    /// Map of key-values was partially changed.
    PartialMap(
        /// List of changes: `(key, change)`.
        Vec<(Intermediate, Change)>,
    ),
    /// Structure with field-value was partially changed.
    PartialStruct(
        /// List of changes: `(field, change)`.
        Vec<(String, Change)>,
    ),
}

impl Change {
    pub fn same() -> Self {
        Self::Same
    }

    pub fn removed() -> Self {
        Self::Removed
    }

    pub fn changed(value: impl Into<Intermediate>) -> Self {
        Self::Changed(value.into())
    }

    pub fn added(value: impl Into<Intermediate>) -> Self {
        Self::Added(value.into())
    }

    pub fn partial_change(change: Self) -> Self {
        Self::PartialChange(Box::new(change))
    }

    pub fn partial_seq() -> Self {
        Self::PartialSeq(vec![])
    }

    pub fn partial_map() -> Self {
        Self::PartialMap(vec![])
    }

    pub fn partial_struct() -> Self {
        Self::PartialStruct(vec![])
    }

    pub fn partial_seq_item(mut self, index: usize, change: Self) -> Self {
        if let Self::PartialSeq(v) = &mut self {
            v.push((index, change));
        }
        self
    }

    pub fn partial_map_item(mut self, key: impl Into<Intermediate>, change: Self) -> Self {
        if let Self::PartialMap(v) = &mut self {
            let key = key.into();
            if let Some(item) = v.iter_mut().find(|(k, _)| k == &key) {
                item.1 = change;
            } else {
                v.push((key, change));
            }
        }
        self
    }

    pub fn partial_struct_item(mut self, name: impl ToString, change: Self) -> Self {
        if let Self::PartialStruct(v) = &mut self {
            let name = name.to_string();
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

    fn optimize(
        self,
        source: &Intermediate,
        target: &Intermediate,
        hint: DiffOptimizationHint,
    ) -> Self {
        match hint {
            DiffOptimizationHint::Default => self,
            DiffOptimizationHint::SizeSource => {
                if self.total_bytesize() > source.total_bytesize() {
                    Self::Changed(target.to_owned())
                } else {
                    self
                }
            }
            DiffOptimizationHint::SizeTarget => {
                if self.total_bytesize() > target.total_bytesize() {
                    Self::Changed(target.to_owned())
                } else {
                    self
                }
            }
            DiffOptimizationHint::SizeValue(threshold) => {
                if self.total_bytesize() > threshold {
                    Self::Changed(target.to_owned())
                } else {
                    self
                }
            }
            DiffOptimizationHint::SizePercentage(threshold) => {
                if self.total_bytesize()
                    > (threshold.clamp(0.0, 1.0) * source.total_bytesize() as f64) as _
                {
                    Self::Changed(target.to_owned())
                } else {
                    self
                }
            }
        }
    }

    pub fn difference(prev: &Intermediate, next: &Intermediate, options: &DiffOptions) -> Self {
        if prev == next {
            Self::Same
        } else {
            match (prev, next) {
                (Intermediate::Option(Some(prev)), Intermediate::Option(Some(next))) => {
                    Self::PartialChange(Box::new(Self::difference(prev, next, options)))
                }
                (Intermediate::NewTypeStruct(prev), Intermediate::NewTypeStruct(next)) => {
                    Self::PartialChange(Box::new(Self::difference(prev, next, options)))
                }
                (
                    Intermediate::NewTypeVariant(prev_name, prev_value),
                    Intermediate::NewTypeVariant(next_name, next_value),
                ) => {
                    if prev_name != next_name {
                        Self::Changed(next.to_owned())
                    } else {
                        Self::PartialChange(Box::new(Self::difference(
                            prev_value, next_value, options,
                        )))
                    }
                }
                (Intermediate::Seq(prev), Intermediate::Seq(next))
                | (Intermediate::Tuple(prev), Intermediate::Tuple(next))
                | (Intermediate::TupleStruct(prev), Intermediate::TupleStruct(next)) => {
                    Self::PartialSeq(Self::sequence_difference(prev, next, options))
                }
                (Intermediate::Map(prev), Intermediate::Map(next)) => {
                    let mut result = vec![];
                    for (nk, nv) in next {
                        if !prev.iter().any(|(pk, _)| pk == nk) {
                            result.push((nk.to_owned(), Self::Added(nv.to_owned())));
                        }
                    }
                    for (pk, _) in prev {
                        if !next.iter().any(|(nk, _)| pk == nk) {
                            result.push((pk.to_owned(), Self::Removed));
                        }
                    }
                    for (pk, pv) in prev {
                        if let Some((_, nv)) = next
                            .iter()
                            .find(|(nk, _)| pk == nk)
                            .filter(|(_, nv)| pv != nv)
                        {
                            let diff = Self::difference(pv, nv, options);
                            if !diff.is_same() {
                                result.push((pk.to_owned(), diff));
                            }
                        }
                    }
                    Self::PartialMap(result)
                }
                (Intermediate::Struct(prev), Intermediate::Struct(next))
                | (Intermediate::StructVariant(_, prev), Intermediate::StructVariant(_, next)) => {
                    let mut result = vec![];
                    for (nk, nv) in next {
                        if !prev.iter().any(|(pk, _)| pk == nk) {
                            result.push((nk.to_owned(), Self::Added(nv.to_owned())));
                        }
                    }
                    for (pk, _) in prev {
                        if !next.iter().any(|(nk, _)| pk == nk) {
                            result.push((pk.to_owned(), Self::Removed));
                        }
                    }
                    for (pk, pv) in prev {
                        if let Some((_, nv)) = next
                            .iter()
                            .find(|(nk, _)| pk == nk)
                            .filter(|(_, nv)| pv != nv)
                        {
                            let diff = Self::difference(pv, nv, options);
                            if !diff.is_same() {
                                result.push((pk.to_owned(), diff));
                            }
                        }
                    }
                    Self::PartialStruct(result)
                }
                _ => Self::Changed(next.to_owned()),
            }
        }
        .optimize(prev, next, options.optimization_hint)
    }

    pub fn sequence_difference(
        prev: &[Intermediate],
        next: &[Intermediate],
        options: &DiffOptions,
    ) -> Vec<(usize, Self)> {
        if prev.is_empty() && next.is_empty() {
            return vec![];
        } else if prev.is_empty() {
            return next
                .iter()
                .enumerate()
                .map(|(i, v)| (i, Self::Added(v.to_owned())))
                .collect();
        } else if next.is_empty() {
            return (0..prev.len()).map(|_| (0, Self::Removed)).collect();
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
                            let (prev_pos, _) = graph.node_weight(chunk[0])?;
                            let prev = &prev[*prev_pos];
                            let next = &next[old_pos];
                            let diff = Self::difference(prev, next, options).optimize(
                                prev,
                                next,
                                options.optimization_hint,
                            );
                            Some((old_pos, diff))
                        }
                        Diff::Removed => Some((old_pos, Self::Removed)),
                        Diff::Added => {
                            pos += 1;
                            Some((old_pos, Self::Added(next[old_pos].to_owned())))
                        }
                    }
                })
                .collect()
        })
        .unwrap_or_default()
    }

    pub fn patch(&self, value: &Intermediate) -> Result<Option<Intermediate>> {
        match self {
            Self::Same => Ok(Some(value.to_owned())),
            Self::Removed => Ok(None),
            Self::Changed(v) => Ok(Some(v.to_owned())),
            Self::Added(_) => Err(Error::CannotAdd(value.to_owned())),
            Self::PartialChange(change) => {
                let result = match value {
                    Intermediate::Option(Some(v)) => match change.patch(v)? {
                        Some(v) => Intermediate::Option(Some(Box::new(v))),
                        _ => return Err(Error::NotPartial(value.to_owned())),
                    },
                    Intermediate::NewTypeStruct(v) => match change.patch(v)? {
                        Some(v) => Intermediate::NewTypeStruct(Box::new(v)),
                        _ => return Err(Error::NotPartial(value.to_owned())),
                    },
                    Intermediate::NewTypeVariant(n, v) => match change.patch(v)? {
                        Some(v) => Intermediate::NewTypeVariant(n.to_owned(), Box::new(v)),
                        _ => return Err(Error::NotPartial(value.to_owned())),
                    },
                    _ => return Err(Error::NotPartial(value.to_owned())),
                };
                Ok(Some(result))
            }
            Self::PartialSeq(changes) => {
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
                                    if let Some(patched) = change.patch(item)? {
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
            Self::PartialMap(changes) => match value {
                Intermediate::Map(v) => {
                    let mut result = v.to_owned();
                    for (key, change) in changes {
                        match change {
                            Self::Removed => {
                                if let Some(index) = result.iter().position(|(k, _)| k == key) {
                                    result.remove(index);
                                }
                            }
                            Self::Changed(v) => {
                                if let Some(index) = result.iter().position(|(k, _)| k == key) {
                                    if let Some(item) = result.get_mut(index) {
                                        item.1 = v.to_owned();
                                    }
                                }
                            }
                            Self::Added(v) => {
                                if let Some(item) = result.iter_mut().find(|(k, _)| k == key) {
                                    item.1 = v.to_owned();
                                } else {
                                    result.push((key.to_owned(), v.to_owned()))
                                }
                            }
                            change => {
                                if let Some(item) = result.iter_mut().find(|(k, _)| k == key) {
                                    if let Some(patched) = change.patch(&item.1)? {
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
            Self::PartialStruct(changes) => {
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
                                    if let Some(patched) = change.patch(&item.1)? {
                                        item.1 = patched;
                                    }
                                }
                            }
                        }
                    }
                    Ok(result)
                }

                match value {
                    Intermediate::Struct(v) => {
                        Ok(Some(Intermediate::Struct(implement(v, changes)?)))
                    }
                    Intermediate::StructVariant(n, v) => Ok(Some(Intermediate::StructVariant(
                        n.to_owned(),
                        implement(v, changes)?,
                    ))),
                    _ => Err(Error::NotMap(value.to_owned())),
                }
            }
        }
    }

    pub fn data_difference<P, N>(prev: &P, next: &N, options: &DiffOptions) -> Result<Self>
    where
        P: Serialize,
        N: Serialize,
    {
        let prev = crate::to_intermediate(prev)?;
        let next = crate::to_intermediate(next)?;
        Ok(Self::difference(&prev, &next, options))
    }

    pub fn data_patch<T>(&self, data: &T) -> Result<Option<T>>
    where
        T: Serialize + DeserializeOwned,
    {
        let serialized = crate::to_intermediate(data)?;
        let patched = match self.patch(&serialized)? {
            Some(patched) => patched,
            None => return Ok(None),
        };
        Ok(Some(crate::from_intermediate::<T>(&patched)?))
    }

    pub fn total_bytesize(&self) -> usize {
        fn string_bytesize(v: &str) -> usize {
            std::mem::size_of_val(v.as_bytes())
        }

        std::mem::size_of_val(self)
            + match self {
                Self::Changed(v) | Self::Added(v) => v.total_bytesize(),
                Self::PartialChange(v) => v.total_bytesize(),
                Self::PartialSeq(v) => v
                    .iter()
                    .map(|(i, v)| std::mem::size_of_val(i) + v.total_bytesize())
                    .sum(),
                Self::PartialMap(v) => v
                    .iter()
                    .map(|(k, v)| k.total_bytesize() + v.total_bytesize())
                    .sum(),
                Self::PartialStruct(v) => v
                    .iter()
                    .map(|(k, v)| string_bytesize(k) + v.total_bytesize())
                    .sum(),
                _ => 0,
            }
    }
}
