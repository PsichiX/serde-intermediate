use rand::prelude::*;
use serde::{Deserialize, Serialize};
use serde_intermediate::*;
use std::collections::HashMap;

pub trait Generate {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: Rng;
}

#[derive(Debug, Clone, Serialize, Deserialize, ReflectIntermediate)]
pub enum LoginMethod {
    Email {
        user: String,
        domain: String,
        password_hash: u64,
    },
    Google(String),
    Twitter(String),
    Facebook(usize),
}

impl Generate for LoginMethod {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: Rng,
    {
        match rng.random_range::<usize, _>(0..4) {
            0 => {
                let users = ["john", "malkovitch", "rocks"];
                let domains = ["gmail.com", "hotmail.com"];
                Self::Email {
                    user: users.choose(rng).unwrap().to_string(),
                    domain: domains.choose(rng).unwrap().to_string(),
                    password_hash: rng.random(),
                }
            }
            1 => {
                let users = ["john", "malkovitch", "rocks"];
                Self::Google(users.choose(rng).unwrap().to_string())
            }
            2 => {
                let users = ["john", "malkovitch", "rocks"];
                Self::Twitter(users.choose(rng).unwrap().to_string())
            }
            3 => Self::Facebook(rng.random::<u64>() as usize),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ReflectIntermediate)]
pub struct Buff {
    multiplier: f32,
    relative: f32,
}

impl Generate for Buff {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: Rng,
    {
        Self {
            multiplier: rng.random_range(0.5..1.5),
            relative: rng.random_range(-2.0..2.0),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ReflectIntermediate)]
pub struct Item {
    id: usize,
    buffs: HashMap<String, Buff>,
}

impl Generate for Item {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: Rng,
    {
        let buff_names = ["attack", "defence", "durability"];
        let count = rng.random_range(0..100);
        Self {
            id: rng.random_range(0..5),
            buffs: std::iter::from_fn(|| {
                Some((
                    buff_names.choose(rng).unwrap().to_string(),
                    Buff::generate(rng),
                ))
            })
            .take(count)
            .collect(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ReflectIntermediate)]
pub struct Inventory {
    coins: usize,
    items: Vec<Item>,
}

impl Generate for Inventory {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: Rng,
    {
        let count = rng.random_range(0..50);
        Self {
            coins: rng.random_range(0..1000000),
            items: std::iter::from_fn(|| Some(Item::generate(rng)))
                .take(count)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ReflectIntermediate)]
pub struct Account {
    user_name: String,
    first_name: Option<String>,
    last_name: Option<String>,
    login: LoginMethod,
    inventory: Inventory,
}

impl Generate for Account {
    fn generate<R>(rng: &mut R) -> Self
    where
        R: Rng,
    {
        let user_names = ["abra", "cadabra", "hocus", "pocus"];
        let first_names = ["Adam", "Eve", "Maria", "Neil"];
        let last_names = ["Not", "Great", "Day", "For", "Being", "Creative"];
        Self {
            user_name: user_names.choose(rng).unwrap().to_string(),
            first_name: if rng.random() {
                Some(first_names.choose(rng).unwrap().to_string())
            } else {
                None
            },
            last_name: if rng.random() {
                Some(last_names.choose(rng).unwrap().to_string())
            } else {
                None
            },
            login: LoginMethod::generate(rng),
            inventory: Inventory::generate(rng),
        }
    }
}
