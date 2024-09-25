
use log::{trace, error};

use near_base::{NearResult, builder_codec_utils::Empty, };
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object};

use crate::process::Process;

// begin
pub struct BeginRoutine {
    process: Process,
}

impl BeginRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(BeginRoutine{
            process
        }))
    }

    #[inline]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for BeginRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("BeginRoutine::on_routine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Empty, req, o, o, { header_meta.sequence() });

        let r: DataContent<u32> = match r {
            DataContent::Content(_) => self.on_routine(header_meta).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl BeginRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta) -> NearResult<u32> {
        dataagent_util::TransactionManager::get_instance()
            .begin_transaction(self.process().db_helper())
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })
    }

}

// commit
pub struct CommitRoutine;

impl CommitRoutine {
    pub fn new() -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(CommitRoutine))
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for CommitRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("CommitRoutine::on_routine: header_meta={header_meta}");

        let r = try_decode_raw_object!(u32, req, o, o, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content(id) => self.on_routine(header_meta, id).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl CommitRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, id: u32) -> NearResult<Empty> {
        dataagent_util::TransactionManager::get_instance()
            .commit(id)
            .await
            .map(| _ | {
                Empty
            })
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })
    }

}


// rollback
pub struct RollbackRoutine;

impl RollbackRoutine {
    pub fn new() -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(RollbackRoutine))
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for RollbackRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("RollbackRoutine::on_routine: header_meta={header_meta}");

        let r = try_decode_raw_object!(u32, req, o, o, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content(id) => self.on_routine(header_meta, id).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl RollbackRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, id: u32) -> NearResult<Empty> {
        dataagent_util::TransactionManager::get_instance()
            .rollback(id)
            .await
            .map(| _ | {
                Empty
            })
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })
    }

}
