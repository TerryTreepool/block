
use log::{trace, error, info};

use near_base::NearResult;
use near_core::time_utils::native_now;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{brand::{Brand_update, Brand_info}, DataContent, try_decode_raw_object, try_encode_raw_object};
use topic_util::types::brand_types::Status;

use crate::{process::Process, p::UPDATE_BRAND, public::CheckTrait, };

pub struct UpdateBrandRoutine {
    process: Process,
}

impl UpdateBrandRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(UpdateBrandRoutine{
            process
        }))
    }

    #[inline]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for UpdateBrandRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("update brand routine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Brand_update, req, o, { o.take_brand() }, { header_meta.sequence() });

        let r: DataContent<Brand_info> = match r {
            DataContent::Content(brand) => self.on_routine(header_meta, brand).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl UpdateBrandRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, brand: Brand_info) -> NearResult<Brand_info> {

        Status::try_from(brand.status())
        .map_err(| e | {
            error!("err = {e}, sequence = {}", header_meta.sequence());
            e
        })?;

        let mut updating_brand =
        crate::public::brand::get_brand(self.process().db_helper(), brand.brand_id())
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        updating_brand.check_status()
            .map_err(| e |{
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        updating_brand.set_status(brand.status());
        updating_brand.set_update_time(native_now().format("%Y-%m-%d %H:%M:%S").to_string());

        self.process()
            .db_helper()
            .execute_with_param(UPDATE_BRAND.0, &updating_brand)
            .await
            .map(| _ | {
                info!("success update {} brand info, sequence = {}", brand.brand_id(), header_meta.sequence());
            })
            .map_err(| e |{
                let error_string = format!("failed update {} brand with err = {e}", brand.brand_id());
                error!("{error_string}, sequence = {}", header_meta.sequence());
                e
            })?;

        Ok(updating_brand)
    }
}
