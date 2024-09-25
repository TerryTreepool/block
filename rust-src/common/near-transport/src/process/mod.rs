
mod itf;
pub mod provider;

pub use itf::{ItfTrait, ItfTraitPtr, ItfBuilderTrait, };
use near_util::TopicRef;
pub use provider::{EventManager, RoutineEventTrait, };

use near_base::*;
use crate::{network::DataContext, package::{PackageHeader, PackageHeaderExt, }, tunnel::DynamicTunnel, HeaderMeta };

pub enum EventResult<Context> {
    Ignore,
    Response(ResponseEvent<Context>),
    Transfer(TransferEvent<Context>),
}

pub struct ResponseEvent<Context> {
    pub data: Context,
}

impl<Context> From<Context> for ResponseEvent<Context> {
    fn from(value: Context) -> Self {
        Self { 
            data: value
        }
    }
}

pub struct TransferEvent<Context> {
    pub to: Vec<(ObjectId, Option<Box<dyn RoutineEventTrait>>)>,
    pub topic: String,
    pub data: Context,
}

impl<Context> TransferEvent<Context> {
    pub fn split(self) -> (Vec<(ObjectId, Option<Box<dyn RoutineEventTrait>>)>, String, Context) {
        (self.to, self.topic, self.data)
    }
}

#[async_trait::async_trait]
pub trait Routine<REQ, RESP>: Send + Sync
where REQ: ItfTrait,
      RESP: ItfTrait {
    async fn on_routine(&self, header_meat: &HeaderMeta, req: REQ) -> EventResult<RESP>;
}

pub struct RoutineWrap<REQ: ItfTrait, RESP: ItfTrait>(Box<dyn Routine<REQ, RESP>>);

impl<REQ: ItfTrait, RESP: ItfTrait> RoutineWrap<REQ, RESP> {
    pub fn new(routine: Box<dyn Routine<REQ, RESP>>) -> Box<dyn RoutineEventTrait> {
        Box::new(Self(routine)) as Box<dyn RoutineEventTrait>
    }
}

pub trait ProcessTrait: Send + Sync {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait>;
    fn create_routine(&self, sender: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>>;
}

pub trait ProcessEventTrait: Send + Sync {
    fn on_reinit(&self);
}

pub(crate) struct EmptyProcessEvent;

impl ProcessEventTrait for EmptyProcessEvent {
    fn on_reinit(&self) {
    }
}

#[async_trait::async_trait]
pub trait PackageEventTrait<DATA>: Send + Sync {

    async fn on_package_event(
        &self, 
        tunnel: DynamicTunnel, 
        head: PackageHeader,
        head_ext: PackageHeaderExt,
        data: DATA,
    ) -> NearResult<()>;

}

#[async_trait::async_trait]
pub trait PackageFailureTrait: Send + Sync {

    async fn on_package_failure(
        &self, 
        error: NearError,
        data: DataContext,
    );

}

#[async_trait::async_trait]
pub trait PackageEstablishedTrait: Send + Sync {
    async fn on_established(
        &self, 
        tunnel: DynamicTunnel
    );
}

// pub struct EmptyProcessEvent;

// #[async_trait::async_trait]
// impl PackageEventTrait for EmptyProcessEvent {

//     async fn on_package_event(
//         &self, 
//         _tunnel: DynamicTunnel, 
//         _head: PackageHeader,
//         _head_ext: PackageHeaderExt,
//         _data: Vec<u8>
//     ) -> NearResult<()> {
//         Ok(())
//     }
// }

