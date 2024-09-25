
pub mod manager;
pub mod cb;
pub mod result;

use std::{str::FromStr, hash::{Hash, Hasher}, collections::{hash_map::DefaultHasher, HashMap}};

use mac_address::MacAddress;
use near_base::{NearError, ErrorCode, utils::make_long, thing::ThingObject, };

use topic_util::types::hci_types::HciTaskId;

use crate::{MAC_ADDRESS, lua::data::Data};

#[derive(Clone, PartialEq, Eq)]
pub enum TaskModule {
    Search,
    AddThing,
    RemoveThing,
    PairThing,
    RemovePairThing,
    QueryThing,
    AnalizeData,
    ControlThing,
    AddSchedule,
    RemoveSchedule,
    ExecuteSchedule,
    Other(String),
}

// impl ToString for TaskModule {
//     fn to_string(&self) -> String {
//         match self {
//             Self::Search => "search_thing".to_owned(),
//             Self::AddThing => "add_thing".to_owned(),
//             Self::RemoveThing => "remove_thing".to_owned(),
//             Self::PairThing => "pair_thing".to_owned(),
//             Self::RemovePairThing => "remove_pair_thing".to_owned(),
//             Self::QueryThing => "query_thing".to_owned(),
//             Self::AnalizeData => "alalize_data".to_owned(),
//             Self::ControlThing => "control_thing".to_owned(),
//             Self::AddSchedule => "add_schedule".to_owned(),
//             Self::RemoveSchedule => "remove_schedule".to_owned(),
//             Self::ExecuteSchedule => "execute_schedule".to_owned(),
//             Self::Other(v) => v.clone(),
//         }
//     }
// }

impl FromStr for TaskModule {
    type Err = NearError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "search_thing" => Ok(Self::Search),
            "add_thing" => Ok(Self::AddThing),
            "remove_thing" => Ok(Self::RemoveThing),
            "pair_thing" => Ok(Self::PairThing),
            "remove_pair_thing" => Ok(Self::RemovePairThing),
            "query_thing" => Ok(Self::QueryThing),
            "analize_data" => Ok(Self::AnalizeData),
            "control_thing" => Ok(Self::ControlThing),
            "add_schedule" => Ok(Self::AddSchedule),
            "remove_schedule" => Ok(Self::RemoveSchedule),
            "execute_schedule" => Ok(Self::ExecuteSchedule),
            "" => Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "task name is null")),
            _ => Ok(Self::Other(s.to_owned())),
        }
    }
}

impl TaskModule {
    pub fn into_value(&self) -> HciTaskId {
        match &self {
            Self::Search => 10,
            Self::AddThing => 11,
            Self::RemoveThing => 12,
            Self::PairThing => 13,
            Self::RemovePairThing => 14,
            Self::QueryThing => 15,
            Self::AnalizeData => 16,
            Self::ControlThing => 17,
            Self::AddSchedule => 21,
            Self::RemoveSchedule => 22,
            Self::ExecuteSchedule => 23,
            Self::Other(v) => {
                let mut hasher = DefaultHasher::new();
                v.hash(&mut hasher);
                let h = (hasher.finish() % (u16::MAX) as u64) as u16;
                make_long(h, 0x1000)
            },
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Search => "search_thing",
            Self::AddThing => "add_thing",
            Self::RemoveThing => "remove_thing",
            Self::PairThing => "pair_thing",
            Self::RemovePairThing => "remove_pair_thing",
            Self::QueryThing => "query_thing",
            Self::AnalizeData => "analize_data",
            Self::ControlThing => "control_thing",
            Self::AddSchedule => "add_schedule",
            Self::RemoveSchedule => "remove_schedule",
            Self::ExecuteSchedule => "execute_schedule",
            Self::Other(v) => v.as_str(),
        }
    }
}

#[derive(Clone)]
pub struct TaskData {
    pub(crate) task_module: TaskModule,
    pub(crate) module_id: String,
    pub(crate) params: HashMap<String, String>,
}

impl From<(TaskModule, &ThingObject)> for TaskData {
    fn from(value: (TaskModule, &ThingObject)) -> Self {
        let (task_module, thing) = value;

        let params = {
            let mut params = thing.body().content().user_data().clone();
            let mut r =[0u8; 6];
            r.clone_from_slice(thing.desc().content().mac_address().as_slice().clone());
            let mac = mac_address::MacAddress::new(r);
            params.insert(MAC_ADDRESS.to_owned(), mac.to_string());
            params
        };

        Self {
            task_module,
            module_id: thing.desc().content().owner_depend_id().to_owned(),
            params,
        }

    }
}

impl From<(TaskModule, ThingObject)> for TaskData {
    fn from(value: (TaskModule, ThingObject)) -> Self {
        let (task_module, mut thing) = value;

        let params = {
            let mut params = thing.mut_body().mut_content().take_userdata();
            let mut r =[0u8; 6];
            r.clone_from_slice(thing.desc().content().mac_address().as_slice().clone());
            let mac = mac_address::MacAddress::new(r);
            params.insert(MAC_ADDRESS.to_owned(), mac.to_string());
            params
        };

        Self {
            task_module,
            module_id: thing.desc().content().owner_depend_id().to_owned(),
            params,
        }

    }
}

pub struct TaskCbData {
    mac: MacAddress,
    data: Data,
}

impl From<(MacAddress, Data)> for TaskCbData {
    fn from(value: (MacAddress, Data)) -> Self {
        let (mac, data) = value;

        Self {
            mac, data,
        }
    }
}

impl TaskCbData {
    pub fn split(self) -> (MacAddress, Data) {
        (self.mac, self.data)
    }
}

#[async_trait::async_trait]
pub trait TaskCbTrait: Send + Sync {
    async fn on_taskcb(&self, task_module: TaskModule, data: TaskCbData);
}
