
use log::{trace, error, info, };

use near_base::{NearResult, ErrorCode, NearError, builder_codec_macro::Empty};
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object};

use crate::process::Process;

pub struct RemoveBrandRoutine {
    process: Process,
}

impl RemoveBrandRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(RemoveBrandRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for RemoveBrandRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("RemoveBrandRoutine: header_meta={header_meta}");

        let r = try_decode_raw_object!(String, req, o, o, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content(brand_id) => self.on_routine(header_meta, brand_id).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl RemoveBrandRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, brand_id: String) -> NearResult<Empty> {

        self.process
            .brand_storage()
            .delete_with_prefix(&brand_id)
            .await
            .map(| _ | {
                info!("Successfully delete [{brand_id}] brand.");
                Empty
            })
            .map_err(| e | {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_NOTFOUND => {
                        let error_string = format!("Not found [{brand_id}] brand.");
                        error!("{error_string}, sequence: {}", header_meta.sequence());
                        NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
                    }
                    _ => {
                        error!("failed delete [{brand_id}] brand with err: {e}, sequence: {}", header_meta.sequence());
                        e
                    }
                }
            })
    }
}
