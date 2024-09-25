
use std::collections::HashMap;

use common::RoutineTemplate;
use log::{trace, error, info};

use near_base::NearResult;
use near_transport::{RoutineEventTrait, RoutineWrap, Routine, HeaderMeta, EventResult};

use base::raw_object::RawObjectGuard;
use protos::{try_decode_raw_object, hci::hci_thing::*, DataContent, try_encode_raw_object};
use topic_util::types::hci_types::HciTaskId;
use topic_util::topics::hci_service::NEAR_THING_SERVICE_CONTROL_THING_PUB;

use crate::process::Process;

pub struct CtrlThingRoutine {
    #[allow(unused)]
    process: Process,
}

impl CtrlThingRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(CtrlThingRoutine{
            process
        }))
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for CtrlThingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("CtrlThingRoutine::on_routine header_meta={header_meta}");

        let r = try_decode_raw_object!(Hci_ctrl_thing, req, o, { (o.take_thing_id(), o.take_data()) }, { header_meta.sequence() });

        let r: DataContent<HciTaskId> = match r {
            DataContent::Content((thing_id, thing_data)) => self.on_routine(header_meta, thing_id, thing_data).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl CtrlThingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, thing_id: String, thing_data: HashMap<String, String>) -> NearResult<HciTaskId> {
        let thing_id_clone = thing_id.clone();
        RoutineTemplate::<HciTaskId>::call_with_headermeta(
            header_meta, 
            NEAR_THING_SERVICE_CONTROL_THING_PUB.topic().clone(),
            vec![(thing_id, thing_data)],
        )
        .await
        .map_err(| e | {
            error!("{e}, sequence: {}", header_meta.sequence());
            e
        })?
        .await
        .map(| task_id | {
            info!("Successfully ctrl [{thing_id_clone}] thing");
            task_id
        })
        .map_err(| e | {
            error!("failed trl [{thing_id_clone}] thing with err: {e}, sequence: {}", header_meta.sequence());
            e
        })

    }
}
