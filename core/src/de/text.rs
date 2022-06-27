use crate::{
    de::intermediate::{deserialize_as, DeserializeMode},
    error::*,
    value::intermediate::Intermediate,
};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use serde::de::DeserializeOwned;

pub fn from_str<T>(value: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    from_str_as(value, Default::default())
}

pub fn from_str_as<T>(value: &str, mode: DeserializeMode) -> Result<T>
where
    T: DeserializeOwned,
{
    let value = intermediate_from_str(value)?;
    deserialize_as(&value, mode)
}

#[derive(Parser)]
#[grammar = "de/text.grammar.pest"]
struct TextParser;

pub fn intermediate_from_str(content: &str) -> Result<Intermediate> {
    let ast = TextParser::parse(Rule::main, content)
        .map_err(|error| Error::Message(format!("{}", error)))?
        .next()
        .ok_or(Error::NoNextTokens)?;
    parse(ast)
}

macro_rules! impl_parse {
    ($variant:ident : $ast:expr) => {{
        let t = $ast.into_inner().next().unwrap().as_str();
        match t.parse() {
            Ok(value) => Ok(Intermediate::$variant(value)),
            Err(_) => Err(Error::CannotParse(t.to_owned())),
        }
    }};
}

fn parse(ast: Pair<Rule>) -> Result<Intermediate> {
    match ast.as_rule() {
        Rule::unit => Ok(Intermediate::Unit),
        Rule::bool => match ast.as_str() {
            "true" => Ok(Intermediate::Bool(true)),
            "false" => Ok(Intermediate::Bool(false)),
            t => Err(Error::InvalidTokens(t.to_owned())),
        },
        Rule::i8 => impl_parse!(I8: ast),
        Rule::i16 => impl_parse!(I16: ast),
        Rule::i32 => impl_parse!(I32: ast),
        Rule::i64 => impl_parse!(I64: ast),
        Rule::i128 => impl_parse!(I128: ast),
        Rule::u8 => impl_parse!(U8: ast),
        Rule::u16 => impl_parse!(U16: ast),
        Rule::u32 => impl_parse!(U32: ast),
        Rule::u64 => impl_parse!(U64: ast),
        Rule::u128 => impl_parse!(U128: ast),
        Rule::f32 => impl_parse!(F32: ast),
        Rule::f64 => impl_parse!(F64: ast),
        Rule::char => impl_parse!(Char: ast),
        Rule::string => Ok(Intermediate::String(
            ast.into_inner().next().unwrap().as_str().to_owned(),
        )),
        Rule::bytes => {
            let t = ast.into_inner().next().unwrap().as_str();
            let bytes = (0..t.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&t[i..(i + 2)], 16))
                .collect::<std::result::Result<Vec<_>, _>>();
            match bytes {
                Ok(bytes) => Ok(Intermediate::Bytes(bytes)),
                Err(_) => Err(Error::CannotParse(t.to_owned())),
            }
        }
        Rule::none => Ok(Intermediate::Option(None)),
        Rule::some => {
            let value = parse(ast.into_inner().next().unwrap())?;
            Ok(Intermediate::Option(Some(Box::new(value))))
        }
        Rule::unit_struct => Ok(Intermediate::UnitStruct),
        Rule::newtype_struct => {
            let value = parse(ast.into_inner().next().unwrap())?;
            Ok(Intermediate::NewTypeStruct(Box::new(value)))
        }
        Rule::seq => {
            let list = ast.into_inner().map(parse).collect::<Result<Vec<_>>>()?;
            Ok(Intermediate::Seq(list))
        }
        Rule::tuple => {
            let list = ast.into_inner().map(parse).collect::<Result<Vec<_>>>()?;
            Ok(Intermediate::Tuple(list))
        }
        Rule::tuple_struct => {
            let list = ast.into_inner().map(parse).collect::<Result<Vec<_>>>()?;
            Ok(Intermediate::TupleStruct(list))
        }
        Rule::map => {
            let pairs = ast
                .into_inner()
                .map(|ast| {
                    let mut pairs = ast.into_inner();
                    let key = parse(pairs.next().unwrap())?;
                    let value = parse(pairs.next().unwrap())?;
                    Ok((key, value))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(Intermediate::Map(pairs))
        }
        Rule::structure => {
            let pairs = ast
                .into_inner()
                .map(|ast| {
                    let mut pairs = ast.into_inner();
                    let key = pairs.next().unwrap().as_str().to_owned();
                    let value = parse(pairs.next().unwrap())?;
                    Ok((key, value))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(Intermediate::Struct(pairs))
        }
        Rule::variant => {
            let mut pairs = ast.into_inner();
            let name = pairs.next().unwrap().as_str().to_owned();
            let content = pairs.next().unwrap();
            match content.as_rule() {
                Rule::unit => Ok(Intermediate::UnitVariant(name)),
                Rule::newtype_struct => Ok(Intermediate::NewTypeVariant(
                    name,
                    Box::new(parse(content)?),
                )),
                Rule::tuple => {
                    let list = content
                        .into_inner()
                        .map(parse)
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Intermediate::TupleVariant(name, list))
                }
                Rule::structure => {
                    let pairs = content
                        .into_inner()
                        .map(|ast| {
                            let mut pairs = ast.into_inner();
                            let key = pairs.next().unwrap().as_str().to_owned();
                            let value = parse(pairs.next().unwrap())?;
                            Ok((key, value))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Intermediate::StructVariant(name, pairs))
                }
                _ => Err(Error::InvalidTokens(content.as_str().to_owned())),
            }
        }
        _ => Err(Error::InvalidTokens(ast.as_str().to_owned())),
    }
}
