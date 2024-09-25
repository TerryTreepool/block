
use base::raw_object::RawObjectGuard;
use log::{trace, error};
use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};
use protos::{hci::brand::Brand_info, DataContent, try_decode_raw_object, try_encode_raw_object};

use crate::process::Process;

pub struct QueryBrandRoutine {
    process: Process,
}

impl QueryBrandRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(Self{
            process,
        }))
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for QueryBrandRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("query a brand routine: header_meta={header_meta}.");

        let r = try_decode_raw_object!(String, req, o, o, { header_meta.sequence() });

        let r: DataContent<Brand_info> = match r {
            DataContent::Content(brand_id) => self.on_routine(header_meta, brand_id).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryBrandRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, brand_id: String) -> NearResult<Brand_info> {

        self.process
            .brand_storage()
            .load_with_prefix(&brand_id)
            .await
            .map(| brand | brand.into())
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })

    }
}
