
use log::{trace, error};

use near_base::{ErrorCode, NearError, NearResult, ObjectId, DeviceObject, ObjectTypeCode, OBJECT_TYPE_DEVICE_CORE, };
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object};

use crate::process::Process;

pub struct CheckOutRoutine {
    process: Process,
}

impl CheckOutRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(CheckOutRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for CheckOutRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("CheckOutRoutine: header_meta={header_meta}");

        let r = try_decode_raw_object!(ObjectId, req, o, o, { header_meta.sequence() });

        let r: DataContent<DeviceObject> = match r {
            DataContent::Content(object_id) => self.on_routine(header_meta, object_id).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl CheckOutRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, object_id: ObjectId) -> NearResult<DeviceObject> {

        match object_id.object_type_code() {
            Ok(code) => {
                if let ObjectTypeCode::Device(v) = code {
                    if v == OBJECT_TYPE_DEVICE_CORE {
                        Ok(())
                    } else {
                        let error_string = format!("[{object_id}] is not core-device.");
                        error!("{}, sequence: {}", error_string, header_meta.sequence());
                        Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, error_string))
                    }
                } else {
                    let error_string = format!("[{object_id}] is not device-id.");
                    error!("{}, sequence: {}", error_string, header_meta.sequence());
                    Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, error_string))
                }
            }
            Err(e) => {
                error!("{e}, sequence: {}", header_meta.sequence());
                Err(e)
            }
        }?;

        self.process
            .device_storage()
            .load_with_prefix(object_id.to_string().as_str())
            .await
            .map(| device | device.take_device())
            .map_err(| e |{
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })
    }

}
