#[cfg(test)]
mod tests;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_intermediate::{error::Result, Intermediate};
use std::{
    any::{type_name, Any, TypeId},
    sync::{Arc, RwLock},
};

lazy_static::lazy_static! {
    static ref FACTORIES: Arc<RwLock<Vec<Factory>>> = Default::default();
}

struct Factory {
    type_tag: &'static str,
    type_id: TypeId,
    construct: fn(&Intermediate) -> Result<Box<dyn Any>>,
}

fn construct<T: DeserializeOwned + 'static>(data: &Intermediate) -> Result<Box<dyn Any>> {
    Ok(Box::new(serde_intermediate::deserialize::<T>(data)?) as Box<dyn Any>)
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TaggedIntermediate {
    type_tag: String,
    #[serde(default)]
    data: Intermediate,
}

impl TaggedIntermediate {
    pub fn register<T>()
    where
        T: Serialize + DeserializeOwned + 'static,
    {
        Self::register_named::<T>(type_name::<T>())
    }

    pub fn register_named<T>(type_tag: &'static str)
    where
        T: Serialize + DeserializeOwned + 'static,
    {
        if let Ok(mut factories) = FACTORIES.write() {
            let type_id = TypeId::of::<T>();
            factories.push(Factory {
                type_tag,
                type_id,
                construct: construct::<T>,
            });
        }
    }

    pub fn unregister<T>()
    where
        T: Serialize + DeserializeOwned + 'static,
    {
        if let Ok(mut factories) = FACTORIES.write() {
            let type_id = TypeId::of::<T>();
            if let Some(index) = factories.iter().position(|f| f.type_id == type_id) {
                factories.remove(index);
            }
        }
    }

    pub fn unregister_all() {
        if let Ok(mut factories) = FACTORIES.write() {
            factories.clear();
        }
    }

    pub fn registered_type_tag<T>() -> Option<&'static str>
    where
        T: 'static,
    {
        if let Ok(factories) = FACTORIES.read() {
            let type_id = TypeId::of::<T>();
            return factories
                .iter()
                .find(|f| f.type_id == type_id)
                .map(|f| f.type_tag);
        }
        None
    }

    pub fn is_registered<T>() -> bool
    where
        T: 'static,
    {
        if let Ok(factories) = FACTORIES.read() {
            let type_id = TypeId::of::<T>();
            return factories.iter().any(|f| f.type_id == type_id);
        }
        false
    }

    pub fn type_tag(&self) -> &str {
        &self.type_tag
    }

    pub fn data(&self) -> &Intermediate {
        &self.data
    }

    pub fn encode<T>(data: &T) -> Result<Self>
    where
        T: Serialize + 'static,
    {
        if let Ok(factories) = FACTORIES.read() {
            let type_id = TypeId::of::<T>();
            if let Some(factory) = factories.iter().find(|f| f.type_id == type_id) {
                return Ok(Self {
                    type_tag: factory.type_tag.to_owned(),
                    data: serde_intermediate::serialize(&data)?,
                });
            }
        }
        Err(serde_intermediate::Error::Message(format!(
            "Factory does not exist for type: {:?}",
            type_name::<T>()
        )))
    }

    pub fn decode_any(&self) -> Result<Box<dyn Any>> {
        if let Ok(factories) = FACTORIES.read() {
            if let Some(factory) = factories.iter().find(|f| f.type_tag == self.type_tag) {
                return (factory.construct)(&self.data);
            }
        }
        Err(serde_intermediate::Error::Message(format!(
            "Factory does not exist for type tag: {:?}",
            self.type_tag
        )))
    }

    pub fn decode<T>(&self) -> Result<T>
    where
        T: 'static,
    {
        self.decode_any()?
            .downcast::<T>()
            .map(|data| *data)
            .map_err(|_| {
                serde_intermediate::Error::Message(format!(
                    "Could not decode value to type: {}",
                    type_name::<T>()
                ))
            })
    }
}
