
use std::{sync::RwLock, collections::{BTreeMap, btree_map::Entry}};

use near_base::{ObjectId, NearResult, NearError, ErrorCode, Timestamp, now};
use once_cell::sync::OnceCell;

use crate::{lua::data::Data, tasks::{TaskCbData, TaskModule}};

pub struct SearchData {
    pub(crate) mac: String,
    pub(crate) dataes: Data,
    pub(crate) timestamp: Timestamp,
}

/// search event result
///////////////////////////////////////////////////////////////////////////////////////
pub struct SeachEventResult {
    result: RwLock<BTreeMap<ObjectId, Vec<SearchData>>>,
}

impl std::default::Default for SeachEventResult {
    fn default() -> Self {
        Self {
            result: RwLock::new(BTreeMap::new()),
        }
    }
}

impl SeachEventResult {
    pub fn get_instance() -> &'static SeachEventResult {
        static INSTANCE: OnceCell<SeachEventResult> = OnceCell::new();

        INSTANCE.get_or_init(|| {
            let r = SeachEventResult::default();
            r
        })
    }

    pub fn add_object(&self, requestor: ObjectId) {
        match self.result
                  .write().unwrap()
                  .entry(requestor) {
            Entry::Occupied(_) => {
            }
            Entry::Vacant(empty) => {
                empty.insert(Vec::new());
            }
        };
    }

    pub fn take(&self, object_id: &ObjectId, count: usize) -> NearResult<Vec<SearchData>> {
        let count = if count == 0 { 10 } else { count };
        let mut dataes = vec![];

        self.result
            .write().unwrap()
            .get_mut(object_id)
            .map(| array | {
                let now = now();
                const ONE_MINUTE: u64 = std::time::Duration::from_secs(60).as_micros() as u64;
                for _ in 0..count {
                    if let Some(a) = array.pop() {
                        if (now - a.timestamp) < ONE_MINUTE {
                            dataes.push(a);
                        }
                    } else {
                        break;
                    }
                }
            })
            .ok_or(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found {object_id} result.")))?;

        Ok(dataes)
    }
}

#[async_trait::async_trait]
impl crate::tasks::TaskCbTrait for SeachEventResult {
    async fn on_taskcb(&self, task_module: TaskModule, data: TaskCbData) {
        debug_assert!(task_module == TaskModule::Search);

        let (mac, dataes) = data.split();

        self.result
            .write().unwrap()
            .values_mut()
            .for_each(| array | {
                array.push(SearchData {
                    mac: mac.to_string(),
                    dataes: dataes.clone(),
                    timestamp: now(),
                })
            });

    }
}
