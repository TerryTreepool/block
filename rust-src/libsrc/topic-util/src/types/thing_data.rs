
use std::{collections::HashMap, ops::Add};

use near_base::ObjectId;

pub type ThingId = ObjectId;

#[derive(Clone)]
pub struct ThingData {
    dataes: HashMap<String, String>,
}

impl std::fmt::Display for ThingData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let debug_string = {
            let mut debug_string = String::default();

            for (k, v) in self.dataes.iter() {
                debug_string = debug_string.add(format!("thing_data[{k}]={v},").as_str());
            }

            debug_string
        };

        write!(f, "{debug_string}")
    }
}

impl From<HashMap<String, String>> for ThingData {
    fn from(value: HashMap<String, String>) -> Self {
        Self{
            dataes: value
        }
    }
}

impl ThingData {
    pub fn take_map(&mut self) -> HashMap<String, String> {
        std::mem::replace(&mut self.dataes, Default::default())
    }

    pub fn clone_map(&self) -> HashMap<String, String> {
        self.dataes.clone()
    }

    pub fn set(&mut self, k: String, v: String) {
        let _ = self.dataes.insert(k, v);
    }

    pub fn get(&self, k: &str) -> Option<&String> {
        self.dataes.get(k)
    }

    pub fn swap(&mut self, mut data: HashMap<String, String>) {
        std::mem::swap(&mut self.dataes, &mut data);
    }

}
