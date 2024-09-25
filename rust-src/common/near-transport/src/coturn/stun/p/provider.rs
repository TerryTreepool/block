
use std::{any::Any, collections::{hash_map::DefaultHasher, BTreeMap}, hash::Hasher, sync::{Arc, RwLock}};

use near_base::{sequence::SequenceString, *};

use crate::{network::DataContext, package::*};

pub struct AnyBaseEventCommand(Box<dyn Any>);

unsafe impl Send for AnyBaseEventCommand {}
unsafe impl Sync for AnyBaseEventCommand {}

impl From<StunReq> for AnyBaseEventCommand {
    fn from(value: StunReq) -> Self {
        Self(Box::new(value))
    }
}

impl<T> AsMut<T> for AnyBaseEventCommand
where T: 'static {

    fn as_mut(&mut self) -> &mut T {
        self.0.downcast_mut::<T>().unwrap()
    }

}

#[async_trait::async_trait]
pub trait BaseEventTrait: Send + Sync {
    async fn emit(
        &self, 
        head: &PackageHeader,
        head_ext: &PackageHeaderExt,
        data: AnyBaseEventCommand,
    ) -> NearResult<()>;

    async fn emit_error(
        &self, 
        error: NearError,
        data: DataContext,
    );
}

struct HasherBuilder<'b> {
    pub(crate) sequence:  &'b SequenceString,
    pub(crate) target: &'b ObjectId,
    pub(crate) timestamp: Timestamp,
}

impl HasherBuilder<'_> {
    pub fn build(self) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write(self.sequence.as_ref());
        hasher.write(self.target.as_ref());
        hasher.write(self.timestamp.to_be_bytes().as_slice());

        hasher.finish()
    }
}

struct EventManagerImpl {
    routines: RwLock<BTreeMap<u64, Box<dyn BaseEventTrait>>>,
}

#[derive(Clone)]
pub struct BaseEventManager(Arc<EventManagerImpl>);

impl BaseEventManager {

    pub fn get_instance() -> &'static BaseEventManager {
        static INSTANCE: once_cell::sync::OnceCell::<BaseEventManager> = once_cell::sync::OnceCell::<BaseEventManager>::new();

        INSTANCE.get_or_init(|| {
            BaseEventManager::new()
        })
    }

    fn new() -> Self {
        Self(Arc::new(EventManagerImpl {
            routines: RwLock::new(BTreeMap::new()),
        }))
    }

    pub fn join_routine(&self, target: &ObjectId, sequence: &SequenceString, timestamp: Timestamp, event: Box<dyn BaseEventTrait>) -> NearResult<()> {
        let routine_id = HasherBuilder{
            target,
            sequence,
            timestamp,
        }.build();

        let routines = &mut *self.0.routines.write().unwrap();

        routines.entry(routine_id)
            .or_insert({
                event
            });

        Ok(())
    }

    pub fn take_routine(&self, target: &ObjectId, sequence: &SequenceString, timestamp: Timestamp) -> Option<Box<dyn BaseEventTrait>> {
        let routine_id = HasherBuilder{
            target,
            sequence,
            timestamp,
        }.build();

        self.0.routines.write().unwrap()
            .remove(&routine_id)
    }

}
