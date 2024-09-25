use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicU64, AtomicU8},
        Arc,
    },
};

use near_base::{now, ErrorCode, NearError, NearResult, Timestamp};

use crate::network::DataContext;

const MESSAGE_CONTEXT_INIT: u8 = 0u8;
const MESSAGE_CONTEXT_LOCK: u8 = 1u8;
const MESSAGE_CONTEXT_FINISHED: u8 = 2u8;

const MESSAGE_STATE_INIT: u8 = 0u8;
const MESSAGE_STATE_FINISHED: u8 = 1u8;

struct MessageContext {
    state: AtomicU8,
    timestamp: AtomicU64,
    data: Arc<Option<DataContext>>,
}

impl MessageContext {
    pub fn new(now: Timestamp) -> Self {
        Self {
            state: AtomicU8::new(MESSAGE_CONTEXT_INIT),
            timestamp: AtomicU64::new(now),
            data: Arc::new(None),
        }
    }

    pub fn with_context(now: Timestamp, data: DataContext) -> Self {
        Self {
            state: AtomicU8::new(MESSAGE_CONTEXT_INIT),
            timestamp: AtomicU64::new(now),
            data: Arc::new(Some(data)),
        }
    }
}

pub enum MessageResult {
    Finished(VecDeque<Option<DataContext>>),
    Wait,
}

pub struct MessageImpl {
    message_created: Timestamp,
    message_timestamp: AtomicU64,
    message_count_finished: AtomicU8,
    message_state: AtomicU8,
    message_count: u8,
    // message: Vec<Arc<MessageContext>>,
    message: Vec<MessageContext>,
}

impl MessageImpl {

    fn push_context(&self, data_context: DataContext) -> NearResult<MessageResult> {
        let head = &data_context.head;
        debug_assert!(head.index() < head.count(), "fatal message");
        debug_assert!(head.index() < self.message_count, "fatal head index");

        let item = self.message.get(head.index() as usize).ok_or_else(|| {
            NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "Index exceeded limit")
        })?;

        let finished = if item
            .state
            .compare_exchange(
                MESSAGE_CONTEXT_INIT,
                MESSAGE_CONTEXT_LOCK,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .is_ok()
        {
            {
                let mut_item =
                    unsafe { &mut *(Arc::as_ptr(&item.data) as *mut Option<DataContext>) };
                *mut_item = Some(data_context);
            }

            item.timestamp.store(now(), std::sync::atomic::Ordering::SeqCst);

            let r = (
                    self.message_count_finished.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
                ) == self.message_count;
            item.state.store(MESSAGE_CONTEXT_FINISHED,std::sync::atomic::Ordering::SeqCst, );

            r
        } else {
            false
        };

        if finished {
            // update message timestamp & state
            self.message_timestamp.store(now(), std::sync::atomic::Ordering::SeqCst);
            self.message_state.store(MESSAGE_STATE_FINISHED, std::sync::atomic::Ordering::SeqCst);

            Ok(MessageResult::Finished({
                self.message
                    .iter()
                    .map(|this| {
                        let mut_data =
                            unsafe { &mut *(Arc::as_ptr(&this.data) as *mut Option<DataContext>) };
                        std::mem::replace(mut_data, None)
                    })
                    .collect()
            }))
        } else {
            Ok(MessageResult::Wait)
        }
    }

    fn push_index(&self, index: u8) -> NearResult<MessageResult> {
        debug_assert!(index < self.message_count, "fatal head index");

        let item = 
            self.message.get(index as usize).ok_or_else(|| {
                NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "Index exceeded limit")
            })?;

        let finished = 
            if item
                .state
                .compare_exchange(
                    MESSAGE_CONTEXT_INIT,
                    MESSAGE_CONTEXT_LOCK,
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                )
                .is_ok() {
                item.timestamp.store(now(), std::sync::atomic::Ordering::SeqCst);
                let r = (self.message_count_finished.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1) == self.message_count;
                item.state.store(MESSAGE_CONTEXT_FINISHED, std::sync::atomic::Ordering::SeqCst, );

                r
            } else {
                false
            };

        if finished {
            // update message timestamp & state
            self.message_timestamp.store(now(), std::sync::atomic::Ordering::SeqCst);
            self.message_state.store(MESSAGE_STATE_FINISHED, std::sync::atomic::Ordering::SeqCst);

            Ok(MessageResult::Finished(Default::default()))
        } else {
            match item.state.load(std::sync::atomic::Ordering::SeqCst) {
                MESSAGE_CONTEXT_FINISHED => Ok(MessageResult::Finished(Default::default())),
                _ => Ok(MessageResult::Wait)
            }
        }
    }

    fn unfinished_context(&self) -> Vec<UnfinishedContext> {
        self.message
            .iter()
            .filter(|message| {
                message.state.load(std::sync::atomic::Ordering::SeqCst) == MESSAGE_CONTEXT_INIT
            })
            .map(|message| {
                UnfinishedContext {
                    message_timestamp: message.timestamp.load(std::sync::atomic::Ordering::SeqCst),
                    message: message.data.clone(),
                }
            })
            .collect()
    }

    #[allow(unused)]
    fn unfinished_index(&self) -> Vec<(Timestamp, u8)> {
        self.message
            .iter()
            .enumerate()
            .filter(|(_, message)| {
                message.state.load(std::sync::atomic::Ordering::SeqCst) == MESSAGE_CONTEXT_INIT
            })
            .map(|(index, data)| {
                (
                    data.timestamp.load(std::sync::atomic::Ordering::SeqCst),
                    index as u8,
                )
            })
            .collect()
    }

    pub fn update_timestamp(&self, index: u8, now: Timestamp) -> NearResult<()> {
        debug_assert!(index < self.message_count, "fatal head index");

        let item = self.message.get(index as usize).ok_or_else(|| {
            NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "Index exceeded limit")
        })?;

        if item.state.load(std::sync::atomic::Ordering::SeqCst) == MESSAGE_CONTEXT_INIT {
            item.timestamp
                .store(now, std::sync::atomic::Ordering::SeqCst);
        }

        Ok(())
    }
}

pub enum Message {
    Vacancy(MessageImpl),
    Occupied(MessageImpl),
}

pub struct UnfinishedContext {
    pub(crate) message_timestamp: Timestamp,
    pub(crate) message: Arc<Option<DataContext>>,
}

impl Message {
    pub fn new(message_count: u8) -> Self {
        let now = now();
        Self::Vacancy(MessageImpl {
            message_created: now,
            message_timestamp: AtomicU64::new(now),
            message_count_finished: AtomicU8::new(0),
            message_state: AtomicU8::new(MESSAGE_STATE_INIT),
            message_count: message_count,
            message: {
                let mut message = vec![];
                for _ in 0..message_count {
                    message.push(MessageContext::new(now));
                }
                message
            },
        })
    }

    pub fn with_message(messages: Vec<DataContext>) -> Self {
        let now = now();
        Self::Occupied(MessageImpl {
            message_created: now,
            message_timestamp: AtomicU64::new(now),
            message_count_finished: AtomicU8::new(0),
            message_state: AtomicU8::new(MESSAGE_STATE_INIT),
            message_count: messages.len() as u8,
            message: messages
                .into_iter()
                .map(|m| MessageContext::with_context(now, m))
                .collect(),
        })
    }

    pub fn push_context(&self, data_context: DataContext) -> NearResult<MessageResult> {
        match self {
            Self::Vacancy(m) => m.push_context(data_context),
            Self::Occupied(_) => Err(NearError::new(
                ErrorCode::NEAR_ERROR_INVALIDPARAM,
                "occupied message can't push.",
            )),
        }
    }

    pub fn push_index(&self, index: u8) -> NearResult<MessageResult> {
        match self {
            Self::Occupied(m) => m.push_index(index),
            Self::Vacancy(_) => Err(NearError::new(
                ErrorCode::NEAR_ERROR_INVALIDPARAM,
                "vacancy message can't push.",
            )),
        }
    }

    pub fn unfinished_context(&self) -> Vec<UnfinishedContext> {
        match self {
            Self::Vacancy(_) => unreachable!("don't reach here"),
            Self::Occupied(m) => m.unfinished_context(),
        }
    }

    #[allow(unused)]
    pub fn unfinished_index(&self) -> Vec<(Timestamp, u8)> {
        match self {
            Self::Vacancy(m) => m.unfinished_index(),
            Self::Occupied(_) => unreachable!("don't reach here"),
        }
    }

    pub fn update_timestamp(&self, indexs: &[u8], now: Timestamp) {
        let m = match self {
            Self::Vacancy(m) => m,
            Self::Occupied(m) => m,
        };

        for index in indexs {
            let _ = m.update_timestamp(*index, now);
        }
    }

    #[inline]
    pub fn message_created_time(&self) -> Timestamp {
        match self {
            Self::Vacancy(m) => m.message_created,
            Self::Occupied(m) => m.message_created,
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_message() {
        use crate::{
            network::DataContext,
            package::{PackageHeader, PackageHeaderExt},
            tunnel::message::Message,
        };

        let data1 = DataContext {
            head: PackageHeader::default()
                .set_major_command(crate::package::MajorCommand::Exchange)
                .set_index(0)
                .set_count(1)
                .set_length(0),
            head_ext: PackageHeaderExt::default(),
            body_data: Default::default(),
        };

        let message = Message::new(1);
        let _r = message.push_context(data1).unwrap();
    }
}
