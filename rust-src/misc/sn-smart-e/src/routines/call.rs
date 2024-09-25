
use log::{trace, error, info};

use near_base::{ErrorCode, NearError, NearResult, builder_codec_macro::Empty, ObjectBuilder, device::{DeviceDescContent, DeviceBodyContent}, };
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use proof::proof_data::{ProofDataSet, ProofOfProcessTrait};
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object};

use crate::process::Process;

pub struct CallRoutine {
    process: Process,
}

impl CallRoutine {

    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(CallRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for CallRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("CallRoutine: header_meta={header_meta}");

        // let r = try_decode_raw_object!(ProofDataSet, req, o, o, { header_meta.sequence() });

        // let r: DataContent<Empty> = match r {
        //     DataContent::Content(proof) => self.on_routine(header_meta, proof).await.into(),
        //     DataContent::Error(e) => DataContent::Error(e),
        // };

        // try_encode_raw_object!(r, { header_meta.sequence() })

        unimplemented!()
    }
}

impl CallRoutine {

    async fn on_routine(&self, header_meta: &HeaderMeta, proof: ProofDataSet) -> NearResult<Empty> {

        Ok(Empty)
    }

}
