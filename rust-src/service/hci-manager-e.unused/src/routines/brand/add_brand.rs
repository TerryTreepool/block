
use log::{trace, error, info};

use near_base::NearResult;
use near_core::time_utils::native_now;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{brand::{Brand_add, Brand_info}, DataContent, try_decode_raw_object, try_encode_raw_object};
use topic_util::types::brand_types::Status;

use crate::{process::Process, 
            routines::brand::BrandIdBuilder};

pub struct AddBrandRoutine {
    process: Process,
}

impl AddBrandRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(AddBrandRoutine{
            process
        }))
    }

    #[inline]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for AddBrandRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("add brand routine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Brand_add, req, body, { body.take_brand() }, { header_meta.sequence() });

        let r: DataContent<Brand_info> = match r {
            DataContent::Content(brand) => self.on_routine(header_meta, brand).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl AddBrandRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, mut brand: Brand_info) -> NearResult<Brand_info> {

        let brand_id = BrandIdBuilder {
            brand_name: brand.brand_name()
        }.build();

        let now = native_now().format("%Y-%m-%d %H:%M:%S").to_string();

        let new_brand = Brand_info {
            brand_id,
            brand_name: brand.take_brand_name(),
            begin_time: now.clone(),
            update_time: now,
            status: Status::Eanbled.into(),
            ..Default::default()
        };

        self.process()
            .db_helper()
            .execute_with_param(crate::p::ADD_BRAND.0, &new_brand)
            .await
            .map(| _ | {
                info!("success add {} brand info, sequence = {}", new_brand.brand_id(), header_meta.sequence());
                new_brand
            })
            .map_err(| e | {
                let error_string = format!("failed add brand with err = {e}");
                error!("{error_string}, sequence = {}", header_meta.sequence());
                e
            })
    }
}
