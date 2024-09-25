
use log::{trace, error};

use near_base::NearResult;
use near_core::time_utils::native_now;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, device::{Device_add, Device_info}, try_decode_raw_object, try_encode_raw_object, };

use crate::{process::Process, public::CheckTrait};

pub struct UpdateDeviceRoutine {
    process: Process,
}

impl UpdateDeviceRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(UpdateDeviceRoutine{
            process
        }))
    }

    #[inline]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for UpdateDeviceRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("update device routine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Device_add, req, o, { o.take_device() }, { header_meta.sequence() });

        let r: DataContent<Device_info> = match r {
            DataContent::Content(device) => self.on_routine(header_meta, device).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl UpdateDeviceRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, mut device: Device_info) -> NearResult<Device_info> {

        let mut mut_device =
            crate::public::thing::get_thing(self.process().db_helper(), device.device_id())
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        mut_device.check_status()
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        mut_device.set_device_name(device.take_device_name());
        mut_device.set_update_time(native_now().format("%Y-%m-%d %H:%M:%S").to_string());

        self.process()
            .db_helper()
            .execute_with_param(crate::p::UPDATE_DEVICE.0, &mut_device)
            .await
            .map(| _ | {
                mut_device
            })
            .map_err(| e |{
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })

    }
}
