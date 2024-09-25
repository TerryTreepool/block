
use log::{trace, error, debug, warn};

use near_base::{ObjectId, NearResult, NearError, ErrorCode};
use near_transport::{RoutineEventTrait, RoutineWrap, Routine, HeaderMeta, EventResult};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, hci::hci_thing::{Hci_thing_list, Hci_thing, hci_thing::Hci_thing_status}, try_encode_raw_object, };
use topic_util::types::hci_types::HciTaskId;

use crate::{process::Process, 
            tasks::{TaskModule, result::search_result::SeachEventResult}, 
            cache::ThingStatus};

pub struct GetTaskResultRoutine {
    process: Process,
}

impl GetTaskResultRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(GetTaskResultRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for GetTaskResultRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("GetTaskResultRoutine::on_routine header_meta={header_meta}");

        let r = match protos::RawObjectHelper::decode::<(HciTaskId, Vec<String>)>(req) {
            Ok(r) => {
                match r {
                    DataContent::Content(c) => DataContent::Content(c),
                    DataContent::Error(_e) => unreachable!()
                }
            }
            Err(e) => {
                let error_string = format!("failed decode message with err={e}");
                log::error!("{error_string} sequence={}", header_meta.sequence());
                protos::DataContent::Error(e)
            }
        };

        let r: DataContent<Hci_thing_list> = match r {
            DataContent::Content((task_id, thing_ids)) => self.on_routine(header_meta, task_id, thing_ids).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl GetTaskResultRoutine {
    async fn on_routine(&self, 
                        header_meta: &HeaderMeta, 
                        task_id: HciTaskId, 
                        thing_ids: Vec<String>) -> NearResult<Hci_thing_list> {
        let creator = 
            header_meta.creator.as_ref().ok_or_else(|| {
                warn!("missing creator. sequence: {}", header_meta.sequence());
                NearError::new(ErrorCode::NEAR_ERROR_NO_TARGET, "missing creator")
            })?
            .creator.as_ref()
            .ok_or_else(|| {
                warn!("missing creator. sequence: {}", header_meta.sequence());
                NearError::new(ErrorCode::NEAR_ERROR_NO_TARGET, "missing creator")
            })?;

        if task_id == TaskModule::Search.into_value() {
            self.get_search_result(header_meta, creator)
        } else if task_id == TaskModule::QueryThing.into_value() {
            self.get_query_result(header_meta, thing_ids).await
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN, format!("undefined task-id {task_id}")))
        }
    }

    fn get_search_result(&self, 
                         header_meta: &HeaderMeta, 
                         creator: &ObjectId) -> NearResult<Hci_thing_list> {
        let r = 
            SeachEventResult::get_instance()
                .take(creator, 0)
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        let mut list = Hci_thing_list::new();
        for a in r {
            let mut one = Hci_thing::new();
            one.set_mac_address(a.mac);
            for (k, v) in a.dataes.take_map() {
                one.mut_data().insert(k, v);
            }
            debug!("get_search_result: mac: {}, thing-data: {:?}", one.mac_address(), one.data());
            list.mut_list().push(one);
        }

        Ok(list)
    }

    async fn get_query_result(&self, 
                              header_meta: &HeaderMeta, 
                              thing_ids: Vec<String>) -> NearResult<Hci_thing_list> {
        let get_thing = | thing_id: &str | {
            self.process
                .thing_components()
                .get_thing_by_id(thing_id)
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })
                .ok()
        };

        let mut thing_list = Hci_thing_list::new();

        for thing_id in thing_ids {
            let (status, data, mac) = 
            if let Some(thing) = get_thing(&thing_id) {
                let mac = mac_address::MacAddress::from(thing.thing().desc().content().mac_address().clone());
                match thing.status() {
                    ThingStatus::Disable => (Hci_thing_status::Disabled, Default::default(), mac),
                    ThingStatus::Offline(_, data) => (Hci_thing_status::Offline, data.clone_map(), mac),
                    ThingStatus::Online(_, data) => (Hci_thing_status::Online, data.clone_map(), mac),
                }
            } else {
                (Hci_thing_status::NotFound, Default::default(), Default::default())
            };

            thing_list.mut_list().push(
                Hci_thing {
                    thing_id,
                    mac_address: mac.to_string(),
                    status: status.into(),
                    data,
                    ..Default::default()
                });
        }

        Ok(thing_list)
    }
}
