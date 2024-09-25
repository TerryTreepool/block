
use std::{sync::RwLock, collections::{BTreeMap, btree_map::Entry}};

use near_base::{NearResult, NearError, ErrorCode};
use once_cell::sync::OnceCell;

use crate::lua::data::Data;


struct ThingResult {
    query_result: Data,
}

/// search event result
///////////////////////////////////////////////////////////////////////////////////////
pub struct ThingEventResult {
    result: RwLock<BTreeMap<String, ThingResult>>,
}

impl std::default::Default for ThingEventResult {
    fn default() -> Self {
        Self {
            result: RwLock::new(BTreeMap::new()),
        }
    }
}

impl ThingEventResult {
    pub fn get_instance() -> &'static ThingEventResult {
        static INSTANCE: OnceCell<ThingEventResult> = OnceCell::new();

        INSTANCE.get_or_init(|| {
            let r = ThingEventResult::default();
            r
        })
    }

    pub fn add_query_result(&self, mac: String, data: Data) {
        let data = data.take_map();
        match self.result
                  .write().unwrap()
                  .entry(mac) {
            Entry::Occupied(exist) => {
                // exist
                exist.get().query_result.merge(data);
            }
            Entry::Vacant(empty) => {
                empty.insert(ThingResult{
                    query_result: data.into(),
                });
            }
        }
    }

    pub fn get_data(&self, mac: &str) -> NearResult<Data> {
        self.result
            .read().unwrap()
            .get(mac)
            .ok_or_else(|| {
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "{mac} not found")
            })
            .map(| data | {
                data.query_result.clone()
            })
    }
}
