
use std::{collections::BTreeMap, sync::{Arc, RwLock}};

use rlua::{AnyUserData, UserData, UserDataMethods};

#[derive(Default)]
struct DataImpl {
    dataes: RwLock<BTreeMap<String, String>>,
}

#[derive(Clone)]
pub struct Data(Arc<DataImpl>);

impl std::default::Default for Data {
    fn default() -> Self {
        Self(Arc::new(
            Default::default()
        ))
    }
}

impl From<BTreeMap<String, String>> for Data {
    fn from(value: BTreeMap<String, String>) -> Self {
        Self(Arc::new(
            DataImpl { dataes: RwLock::new(value), }
        ))
    }
}

impl Data {
    pub fn into_map(&self) -> BTreeMap<String, String> {
        self.0.dataes
            .read().unwrap()
            .clone()
    }

    pub fn set(&self, k: String, v: String) {
        let _ = 
        self.0
            .dataes
            .write().unwrap()
            .insert(k, v);
    }

    pub fn get(&self, k: &str) -> Option<String> {
        self.0
            .dataes
            .read().unwrap()
            .get(k)
            .cloned()
    }
}

impl UserData for Data {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("set", |_, (ud, k, v): (AnyUserData, String, String)| {
            // ud.borrow_mut::<Data>()?.set(String::from_utf8_lossy(k.as_slice()).to_string(), v);
            ud.borrow_mut::<Data>()?.set(k, v);
            Ok(())
        });

        methods.add_function("get", |_, (ud, k): (AnyUserData, String)| {
            // let r = ud.borrow::<Data>()?.get(String::from_utf8_lossy(k.as_slice()).to_string().as_str()).unwrap_or_default();
            let r = ud.borrow::<Data>()?.get(&k).unwrap_or_default();
            Ok(r)
        });

    }
}
