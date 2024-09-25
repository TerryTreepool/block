
use std::{ops::Deref, sync::{Mutex, Arc}, };

use async_std::channel::Sender;
use log::{trace, info, error};

use base::{raw_object::RawObjectGuard};

use near_base::ObjectId;
use near_transport::{Routine, RoutineEventTrait, RoutineWrap, EventResult, HeaderMeta};
use protos::{brand::{Brand_query_resp, Brand_info}, Empty, DataContent};

/// query all brand
struct QueryAllBrandCbImpl {
    snd: Sender<bool>,
    array: Mutex<Vec<Brand_info>>,
}
#[derive(Clone)]
pub struct QueryAllBrandCb(Arc<QueryAllBrandCbImpl>);

impl QueryAllBrandCb {
    pub fn new(snd: Sender<bool>) -> Box<Self> {
        Box::new(Self(Arc::new(QueryAllBrandCbImpl {
            snd,
            array: Mutex::new(vec![]),
        })))
    }

    pub fn take_result(&self) -> Vec<Brand_info> {
        let mut array = vec![];
        let mut_array = &mut *self.0.array.lock().unwrap();
        std::mem::swap(mut_array, &mut array);

        array
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for QueryAllBrandCb {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("QueryAllBrandCb::on_routine, header_meta={}, req={}", header_meta, req.deref());

        let mut array = 
            match protos::RawObjectHelper::decode::<Brand_query_resp>(req) {
                Ok(resp) => {
                    match resp {
                        DataContent::Content(mut brand_list) => {
                            Some(brand_list.take_brands())
                        }
                        DataContent::Error(e) => {
                            error!("response error= {e}, sequence = {}", header_meta.sequence());
                            None
                        }
                    }
                }
                Err(e) => {
                    error!("failed decode message with err = {e}, sequence = {}", header_meta.sequence());
                    None
                }
            };

        if let Some(array) = array.as_mut() {
            let mut_array = &mut *self.0.array.lock().unwrap();
            std::mem::swap(mut_array, array);
        }

        if let Err(e) = self.0.snd.send(array.is_some()).await {
            let error_string = format!("failed send channel with err = {e}, sequence = {}", header_meta.sequence());
            error!("{error_string}");
        }

        EventResult::Ignore
    }
}

/// query brand
struct QueryBrandCbImpl {
    snd: Sender<bool>,
    brand_info: Mutex<Brand_info>,
}
pub struct QueryBrandCb(Arc<QueryBrandCbImpl>);

impl QueryBrandCb {
    pub fn new(snd: Sender<bool>) -> Box<Self> {
        Box::new(Self(Arc::new(
            QueryBrandCbImpl {
                snd, 
                brand_info: Mutex::new(Default::default())
            }
        )))
    }

    pub fn take_result(&self) -> Brand_info {
        let mut result = Default::default();
        let brand_info = &mut *self.0.brand_info.lock().unwrap();
        std::mem::swap(&mut result, brand_info);
        result
    }
}

/// add brand
pub struct AddBrandCb {
    snd: Sender<bool>,
}

impl AddBrandCb {
    pub fn new(snd: Sender<bool>) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(AddBrandCb{ snd }))
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for AddBrandCb {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("AddBrandCb::on_routine, header_meta={}, req={}", header_meta, req.deref());

        let r = 
        match protos::RawObjectHelper::decode::<Empty>(req) {
            Ok(_) => {
                true
            },
            Err(e) => {
                info!("failed add brand with err = {e}, sequence = {}", header_meta.sequence());
                false
            }
        };

        if let Err(e) = self.snd.send(r).await {
            let error_string = format!("failed send channel with err = {e}, sequence = {}", header_meta.sequence());
            error!("{error_string}");
        }

        EventResult::Ignore
    }
}
