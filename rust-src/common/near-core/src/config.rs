use std::{path::PathBuf,
          sync::RwLock, collections::{BTreeMap, btree_map::Entry},
    };
use super::Value;

pub struct Config<'a>(RwLock<BTreeMap<&'a str, Value<'a>>>);

impl<'a> Config<'a> {
    pub fn new() -> Self {
        Self(RwLock::new(BTreeMap::new()))
    }

    pub fn get(&self, name: &'a str) -> Option<&Value<'a>> {
        self.0.read().unwrap()
            .get(name)
    }

    pub fn set(&self, name: &'a str, value: Value<'a>) {
        match self.0.write().unwrap()
                  .entry(name) {
            Entry::Occupied(mut o) => {
                o.insert(value);
            }
            Entry::Vacant(v) => {
                v.insert(value);
            }
        }
    }
}

lazy_static::lazy_static! {
    pub static ref MainConfig: Config<'static> = Config::new();
}
