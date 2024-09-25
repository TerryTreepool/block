
use common::RoutineTemplate;
use log::{trace, error};

use near_base::NearResult;
use near_transport::{RoutineEventTrait, RoutineWrap, Routine, HeaderMeta, EventResult};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, try_decode_raw_object, hci::brand::Brand_info};
use topic_util::{types::hci_types::HciTaskId, 
                 topics::{hci_service::NEAR_THING_SERVICE_SEARCH_PUB, hci_storage::NEAR_THING_STORAGE_BRAND_QUERY_PUB}};

use crate::process::Process;

pub struct SearchRoutine {
    process: Process,
}

impl SearchRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(SearchRoutine{
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
impl Routine<RawObjectGuard, RawObjectGuard> for SearchRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("SearchRoutine::on_routine header_meta={header_meta}");

        let r = try_decode_raw_object!(String, req, c, c, { header_meta.sequence() });

        let r: DataContent<HciTaskId> = match r {
            DataContent::Content(brand_id) => {
                self.on_routine(header_meta, brand_id)
                    .await
                    .into()
            }
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl SearchRoutine {

    pub(in self) async fn on_routine(&self, header_meta: &HeaderMeta, brand_id: String) -> NearResult<HciTaskId> {
        // query brand
        let brand = 
            RoutineTemplate::<Brand_info>::call_with_headermeta(
                header_meta, 
                NEAR_THING_STORAGE_BRAND_QUERY_PUB.topic().clone(),
                brand_id
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
            })?;

        RoutineTemplate::<HciTaskId>::call_with_headermeta(
            header_meta,
            NEAR_THING_SERVICE_SEARCH_PUB.topic().clone(),
            brand.brand_id
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
