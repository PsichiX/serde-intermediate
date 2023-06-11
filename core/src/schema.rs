use pest::{iterators::Pairs, Parser};
use pest_derive::Parser;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

#[derive(Parser)]
#[grammar = "schema.grammar.pest"]
struct SchemaIdParser;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SchemaIdTree {
    Tuple(Vec<Self>),
    Path { path: Vec<String>, args: Vec<Self> },
}

impl SchemaIdTree {
    pub fn new<T>() -> Self {
        let id = SchemaId::new::<T>();
        id.tree()
            .unwrap_or_else(|| panic!("Cannot produce schema id tree from: {}", id.id()))
    }

    pub fn as_tuple(&self) -> Option<&[Self]> {
        match self {
            Self::Tuple(list) => Some(list),
            _ => None,
        }
    }

    pub fn as_path(&self) -> Option<&[String]> {
        match self {
            Self::Path { path, .. } => Some(path),
            _ => None,
        }
    }

    pub fn as_path_name(&self) -> Option<&str> {
        self.as_path()
            .and_then(|list| list.last())
            .map(|segment| segment.as_str())
    }

    pub fn as_path_args(&self) -> Option<&[Self]> {
        match self {
            Self::Path { args, .. } => Some(args),
            _ => None,
        }
    }
}

impl std::fmt::Display for SchemaIdTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tuple(list) => {
                write!(f, "(")?;
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    item.fmt(f)?;
                }
                write!(f, ")")?;
            }
            Self::Path { path, args } => {
                for (i, segment) in path.iter().enumerate() {
                    if i > 0 {
                        write!(f, "::")?;
                    }
                    segment.fmt(f)?;
                }
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        arg.fmt(f)?;
                    }
                    write!(f, ">")?;
                }
            }
        }
        Ok(())
    }
}

impl TryFrom<SchemaId> for SchemaIdTree {
    type Error = ();

    fn try_from(id: SchemaId) -> Result<Self, Self::Error> {
        id.tree().ok_or(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SchemaId(String);

impl SchemaId {
    pub fn new<T>() -> Self {
        Self(std::any::type_name::<T>().to_string())
    }

    pub fn id(&self) -> &str {
        &self.0
    }

    pub fn tree(&self) -> Option<SchemaIdTree> {
        let pairs = SchemaIdParser::parse(Rule::main, &self.0)
            .ok()?
            .next()?
            .into_inner();
        Some(Self::parse_tree(pairs))
    }

    fn parse_tree(mut pairs: Pairs<Rule>) -> SchemaIdTree {
        let pair = pairs.next().unwrap();
        match pair.as_rule() {
            Rule::tuple_element => SchemaIdTree::Tuple(Self::parse_list(pair.into_inner())),
            Rule::path_element => {
                let mut pairs = pair.into_inner();
                let path = Self::parse_path(pairs.next().unwrap().into_inner());
                let args = pairs
                    .next()
                    .map(|pair| Self::parse_list(pair.into_inner()))
                    .unwrap_or_default();
                SchemaIdTree::Path { path, args }
            }
            _ => unreachable!(),
        }
    }

    fn parse_path(pairs: Pairs<Rule>) -> Vec<String> {
        pairs.map(|pair| pair.as_str().to_owned()).collect()
    }

    fn parse_list(pairs: Pairs<Rule>) -> Vec<SchemaIdTree> {
        pairs
            .map(|pair| Self::parse_tree(pair.into_inner()))
            .collect()
    }
}

impl std::fmt::Display for SchemaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<SchemaIdTree> for SchemaId {
    fn from(tree: SchemaIdTree) -> Self {
        Self(tree.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SchemaIdContainer {
    Id(SchemaId),
    Tree(SchemaIdTree),
}

impl SchemaIdContainer {
    pub fn new<T>(prefer_tree: bool) -> Self {
        let id = SchemaId::new::<T>();
        if prefer_tree {
            id.tree()
                .map(|tree| tree.into())
                .unwrap_or_else(|| id.into())
        } else {
            id.into()
        }
    }

    pub fn new_id(id: impl Into<SchemaId>) -> Self {
        Self::Id(id.into())
    }

    pub fn new_tree(tree: impl Into<SchemaIdTree>) -> Self {
        Self::Tree(tree.into())
    }

    pub fn as_id(&self) -> Option<&SchemaId> {
        match self {
            Self::Id(id) => Some(id),
            _ => None,
        }
    }

    pub fn as_tree(&self) -> Option<&SchemaIdTree> {
        match self {
            Self::Tree(tree) => Some(tree),
            _ => None,
        }
    }

    pub fn into_id(self) -> SchemaId {
        match self {
            Self::Id(id) => id,
            Self::Tree(tree) => tree.into(),
        }
    }

    pub fn try_into_tree(self) -> Option<SchemaIdTree> {
        match self {
            Self::Id(id) => id.tree(),
            Self::Tree(tree) => Some(tree),
        }
    }
}

impl std::fmt::Display for SchemaIdContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::Tree(tree) => tree.fmt(f),
        }
    }
}

impl From<SchemaId> for SchemaIdContainer {
    fn from(id: SchemaId) -> Self {
        Self::Id(id)
    }
}

impl From<SchemaIdTree> for SchemaIdContainer {
    fn from(tree: SchemaIdTree) -> Self {
        Self::Tree(tree)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaTypeInstance {
    pub id: SchemaIdContainer,
    #[serde(default)]
    pub description: String,
}

impl SchemaTypeInstance {
    pub fn new(id: impl Into<SchemaIdContainer>) -> Self {
        Self {
            id: id.into(),
            description: Default::default(),
        }
    }

    pub fn description(mut self, content: impl ToString) -> Self {
        self.description = content.to_string();
        self
    }
}

impl<ID> From<ID> for SchemaTypeInstance
where
    ID: Into<SchemaIdContainer>,
{
    fn from(id: ID) -> Self {
        Self::new(id)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaTypeTuple(pub Vec<SchemaTypeInstance>);

impl SchemaTypeTuple {
    pub fn item(mut self, type_instance: impl Into<SchemaTypeInstance>) -> Self {
        self.0.push(type_instance.into());
        self
    }
}

impl<TI> FromIterator<TI> for SchemaTypeTuple
where
    TI: Into<SchemaTypeInstance>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = TI>,
    {
        Self(
            iter.into_iter()
                .map(|type_instance| type_instance.into())
                .collect(),
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaTypeStruct(pub HashMap<String, SchemaTypeInstance>);

impl SchemaTypeStruct {
    pub fn field(
        mut self,
        name: impl ToString,
        type_instance: impl Into<SchemaTypeInstance>,
    ) -> Self {
        self.0.insert(name.to_string(), type_instance.into());
        self
    }
}

impl<N, TI> FromIterator<(N, TI)> for SchemaTypeStruct
where
    N: ToString,
    TI: Into<SchemaTypeInstance>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (N, TI)>,
    {
        Self(
            iter.into_iter()
                .map(|(name, type_instance)| (name.to_string(), type_instance.into()))
                .collect(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaTypeArrayOrSlice {
    pub type_instance: SchemaTypeInstance,
    pub count: usize,
}

impl SchemaTypeArrayOrSlice {
    pub fn new(type_instance: impl Into<SchemaTypeInstance>, count: usize) -> Self {
        Self {
            type_instance: type_instance.into(),
            count,
        }
    }
}

impl<TI> From<(TI, usize)> for SchemaTypeArrayOrSlice
where
    TI: Into<SchemaTypeInstance>,
{
    fn from((type_instance, count): (TI, usize)) -> Self {
        Self::new(type_instance, count)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchemaTypeEnumVariant {
    #[default]
    Empty,
    Tuple(SchemaTypeTuple),
    Struct(SchemaTypeStruct),
}

impl SchemaTypeEnumVariant {
    pub fn new_tuple(content: impl Into<SchemaTypeTuple>) -> Self {
        Self::Tuple(content.into())
    }

    pub fn new_struct(content: impl Into<SchemaTypeStruct>) -> Self {
        Self::Struct(content.into())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaTypeEnum(pub HashMap<String, SchemaTypeEnumVariant>);

impl SchemaTypeEnum {
    pub fn variant(
        mut self,
        name: impl ToString,
        content: impl Into<SchemaTypeEnumVariant>,
    ) -> Self {
        self.0.insert(name.to_string(), content.into());
        self
    }
}

impl<N, V> FromIterator<(N, V)> for SchemaTypeEnum
where
    N: ToString,
    V: Into<SchemaTypeEnumVariant>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (N, V)>,
    {
        Self(
            iter.into_iter()
                .map(|(name, variant)| (name.to_string(), variant.into()))
                .collect(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchemaType {
    Tuple(SchemaTypeTuple),
    Array(SchemaTypeArrayOrSlice),
    Slice(SchemaTypeArrayOrSlice),
    TupleStruct(SchemaTypeTuple),
    Struct(SchemaTypeStruct),
    Enum(SchemaTypeEnum),
}

impl SchemaType {
    pub fn new_tuple(content: impl Into<SchemaTypeTuple>) -> Self {
        Self::Tuple(content.into())
    }

    pub fn new_array(content: impl Into<SchemaTypeArrayOrSlice>) -> Self {
        Self::Array(content.into())
    }

    pub fn new_slice(content: impl Into<SchemaTypeArrayOrSlice>) -> Self {
        Self::Slice(content.into())
    }

    pub fn new_tuple_struct(content: impl Into<SchemaTypeTuple>) -> Self {
        Self::TupleStruct(content.into())
    }

    pub fn new_struct(content: impl Into<SchemaTypeStruct>) -> Self {
        Self::Struct(content.into())
    }

    pub fn new_enum(content: impl Into<SchemaTypeEnum>) -> Self {
        Self::Enum(content.into())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    pub data_type: SchemaType,
    #[serde(default)]
    pub description: String,
}

impl Schema {
    pub fn new(data_type: impl Into<SchemaType>) -> Self {
        Self {
            description: Default::default(),
            data_type: data_type.into(),
        }
    }

    pub fn description(mut self, content: impl ToString) -> Self {
        self.description = content.to_string();
        self
    }
}

impl<T> From<T> for Schema
where
    T: Into<SchemaType>,
{
    fn from(data_type: T) -> Self {
        Self::new(data_type)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaPackage {
    #[serde(default)]
    pub prefer_tree_id: bool,
    pub schemas: HashMap<SchemaIdContainer, Schema>,
}

impl SchemaPackage {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            prefer_tree_id: false,
            schemas: HashMap::with_capacity(capacity),
        }
    }

    pub fn prefer_tree_id(mut self, value: bool) -> Self {
        self.prefer_tree_id = value;
        self
    }

    pub fn with(
        &mut self,
        id: impl Into<SchemaIdContainer>,
        schema: impl Into<Schema>,
    ) -> &mut Self {
        self.schemas.insert(id.into(), schema.into());
        self
    }
}

pub trait SchemaIntermediate: Sized {
    fn schema(package: &mut SchemaPackage) -> SchemaIdContainer;
}
