
use std::str::FromStr;

use log::{trace, error, warn};

use near_base::{ErrorCode, NearError, NearResult, device::DeviceId, DeviceObject, now, };
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object};

use crate::{process::Process, caches::peer_manager::FindPeerReason};

pub struct CheckoutRoutine {
    process: Process,
}

impl CheckoutRoutine {

    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(CheckoutRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for CheckoutRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("CheckoutRoutine: header_meta={header_meta}");

        let r = try_decode_raw_object!(String, req, o, o, { header_meta.sequence() });

        let r: DataContent<DeviceObject> = match r {
            DataContent::Content(peer_id) => self.on_routine(header_meta, peer_id).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl CheckoutRoutine {

    async fn on_routine(&self, header_meta: &HeaderMeta, peer_id: String) -> NearResult<DeviceObject> {

        let peer_id = 
            DeviceId::from_str(&peer_id).map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        let peer = 
            self.process
                .peer_manager()
                .find_peer(&peer_id, Some(FindPeerReason::Checkout(now())))
                .ok_or_else(|| {
                    let error_string = format!("not found [{}] peer.", peer_id);
                    warn!("{error_string}, sequence: {}", header_meta.sequence());
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
                })?;

        Ok(peer.desc)
    }

}
