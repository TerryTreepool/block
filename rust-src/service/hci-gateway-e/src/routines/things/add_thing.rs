
use common::RoutineTemplate;
use log::{trace, error};

use near_base::{NearResult, thing::ThingObject};
use near_transport::{Routine, HeaderMeta, RoutineWrap, RoutineEventTrait, EventResult};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, hci::hci_thing::*, try_encode_raw_object, hci::thing::{Thing_add, Thing_info}};
use topic_util::{topics::{hci_service::NEAR_THING_SERVICE_ADD_THING_PUB, hci_storage::NEAR_THING_STORAGE_THING_ADD_PUB}, 
                 types::hci_types::HciTaskId};

use crate::process::Process;

pub struct AddThingRoutine {
    #[allow(unused)]
    process: Process,
}

impl AddThingRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(AddThingRoutine{
            process
        }))
    }

    #[inline]
    #[allow(unused)]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for AddThingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("AddThingRoutine::on_routine header_meta={header_meta}");

        let r = try_decode_raw_object!(Hci_add_thing, req, o, o, { header_meta.sequence() });

        let r: DataContent<HciTaskId> = match r {
            DataContent::Content(c) => {
                self.on_routine(header_meta, c)
                    .await
                    .into()
            },
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl AddThingRoutine {
    pub(in self) async fn on_routine(&self, header_meta: &HeaderMeta, mut hci_add_thing: Hci_add_thing) -> NearResult<HciTaskId> {

        // todo!
        // check creator right

        let thing = {
            let brand_id = hci_add_thing.take_brand_id();
            let major_product_id = hci_add_thing.take_major_product_id();
            let minor_product_id = hci_add_thing.take_minor_product_id();
            let thing_name = hci_add_thing.take_thing_name();
            let mut thing = hci_add_thing.take_thing();

            RoutineTemplate::<ThingObject>::call_with_headermeta(
                header_meta,
                NEAR_THING_STORAGE_THING_ADD_PUB.topic().clone(),
                Thing_add {
                    thing: Some(Thing_info {
                        brand_id,
                        major_product_id,
                        minor_product_id,
                        thing_name,
                        mac_address: thing.take_mac_address(),
                        ..Default::default()
                    }).into(),
                    thing_data: thing.take_data(),
                    ..Default::default()
                }
            )
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?
        };

        RoutineTemplate::<HciTaskId>::call_with_headermeta(
            header_meta, 
            NEAR_THING_SERVICE_ADD_THING_PUB.topic().clone(),
            thing
        )
        .await
        .map_err(| e | {
            error!("{e}, sequence: {}", header_meta.sequence());
            e
        })?
        .await
        .map_err(| e | {
            error!("{e}, sequence: {}", header_meta.sequence());
            e
        })

    }

}