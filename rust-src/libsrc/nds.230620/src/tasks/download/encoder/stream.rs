
use std::{sync::{atomic::{AtomicU16, AtomicBool, Ordering}, RwLock, Mutex}, };

use log::error;
use near_base::{StateWaiter, ChunkId, NearResult, NearError, ErrorCode, };

use crate::{nds_protocol::{PieceMessage, PieceEncodeDesc}, inc::{ChunkWriterTrait, ChunkWriterFeedbackTrait}, tasks::download::h::OnEventTrait};

use super::FeedbackState;

struct IndexQueue {
    count: AtomicU16,
    queue: Vec<AtomicBool>,
}

impl IndexQueue {
    pub fn new(chunk: &ChunkId) -> Self {
        let chunk_len = chunk.len();
        let piece_cnt = if chunk_len % PieceMessage::payload_max_len() == 0 {
            chunk_len / PieceMessage::payload_max_len()
        } else {
            chunk_len / PieceMessage::payload_max_len() + 1
        };

        Self {
            count: AtomicU16::new(0),
            queue: {
                let mut queue = vec![];
                for _ in 0..piece_cnt {
                    queue.push(AtomicBool::new(false));
                }
                queue
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        self.count.load(Ordering::SeqCst) as usize == self.queue.len()
    }

    pub fn check_and_point(&self, desc: &PieceEncodeDesc) -> bool {
        match desc {
            PieceEncodeDesc::Range(index, _) => {
                if let Some(q) = self.queue.get(*index as usize) {
                    if let Ok(_) = q.compare_exchange(false, 
                                                      true, 
                                                      Ordering::SeqCst, 
                                                      Ordering::Acquire) {
                        self.count.fetch_add(1, Ordering::SeqCst);
                        true
                    } else {
                        false
                    }
                } else {
                    unreachable!("fatal error, invalid index.")
                }    
            }
        }
    }
}

pub struct ChunkPieceRangeEncoder {
    indices: IndexQueue,
    writer: Box<dyn ChunkWriterTrait>,
    waiters: RwLock<StateWaiter>,
    state: Mutex<FeedbackState>,
    pending: AtomicBool,
}

impl ChunkPieceRangeEncoder {
    pub fn new<W: ChunkWriterTrait>(chunk: &ChunkId, writer: W) -> Self {
        Self {
            indices: IndexQueue::new(chunk),
            writer: writer.clone_as_writer(),
            waiters: RwLock::new(StateWaiter::new()),
            state: Mutex::new(FeedbackState::Pending),
            pending: AtomicBool::new(true),
        }
    }
}

impl ChunkPieceRangeEncoder {
    pub async fn wait_finished(&self) -> FeedbackState {
        let waiter = { self.waiters.write().unwrap().new_waiter() };

        StateWaiter::wait(waiter, || {
            self.state.lock().unwrap().clone()
        }).await
    }

    pub(self) fn wake(&self) {
        let waker = { self.waiters.write().unwrap().transfer() };

        waker.wake();
    }

}

#[async_trait::async_trait]
impl ChunkWriterFeedbackTrait for ChunkPieceRangeEncoder {
    async fn finished(&self, writer: Box<dyn ChunkWriterTrait>) {
        {
            let state = &mut *self.state.lock().unwrap();

            match state {
                FeedbackState::Error(_) => { unreachable!("Impossible process") }
                FeedbackState::Finished => { /* ingnore */ }
                FeedbackState::Pending => {
                    *state = FeedbackState::Finished;
                }
            }    
        }

        self.wake();
    }

    async fn err(&self, e: NearError) {
        {
            let state = &mut *self.state.lock().unwrap();

            match state {
                FeedbackState::Error(_) => { /* ignore */ }
                FeedbackState::Finished => { unreachable!("Impossible process") }
                FeedbackState::Pending => {
                    *state = FeedbackState::Error(e);
                }
            }
        }

        self.wake();
    }
}

#[async_trait::async_trait]
impl OnEventTrait for ChunkPieceRangeEncoder {

    async fn on_piece_data(&self, data: &PieceMessage) -> NearResult<()> {
        if !self.pending.load(Ordering::SeqCst) {
            match &*self.state.lock().unwrap() {
                FeedbackState::Pending => { unreachable!("Impossible error") }
                FeedbackState::Finished => { return Ok(()); }
                FeedbackState::Error(e) => { return Err(e.clone()); }
            }
        }

        if !self.indices.check_and_point(&data.desc) {
            Err(NearError::new(ErrorCode::NEAR_ERROR_OPERATOR_COMPLETED, "completed"))
        } else {
            let (index, _) = match data.desc.to_range() {
                Some(range) => { Ok(range) }
                None => {
                    let error_string = format!("Encoding format error, it isn't Range.");
                    error!("{}", error_string);
                    let err = NearError::new(ErrorCode::NEAR_ERROR_ENCODING_FORMAT, error_string);
                    self.err(err.clone()).await;
                    Err(err)
                }
            }?;

            match self.writer.write(&data.chunk, index as usize * PieceMessage::payload_max_len(), &data.data).await {
                Ok(_) => {
                    if self.indices.is_finished() {
                        self.finished(self.writer.clone_as_writer()).await;
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("failed to write with err = {}", e);
                    self.err(e.clone());
                    Err(e)
                }
            }
        }
    }

}
