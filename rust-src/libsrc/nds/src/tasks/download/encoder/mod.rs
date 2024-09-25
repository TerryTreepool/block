
mod stream;

use near_base::{NearError, };
pub use stream::ChunkPieceRangeEncoder;

use crate::inc::ChunkWriterTrait;

pub enum FeedbackState {
    Pending,
    Finished,
    Error(NearError),
}

impl Clone for FeedbackState {
    fn clone(&self) -> Self {
        match self {
            FeedbackState::Pending => Self::Pending,
            FeedbackState::Finished => Self::Finished,
            FeedbackState::Error(e) => Self::Error(e.clone()),
        }
    }
}
