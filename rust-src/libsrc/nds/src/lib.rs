
mod stack;
mod stack_private;
mod nds_protocol;
mod inc;
mod tasks;
mod chunks;
mod statistic;
// pub mod topic_routine;

use near_base::{NearError, NearResult};
pub use stack::{Stack as NdsStack, Config as NdsConfig};
pub use tasks::{DownloadSource, SingleDownloadSource, MultiDownloadSource};

pub enum NdsState {
    Prepair(near_base::file::FileObject),
    Pending(statistic::BytePerfStatPtr),
    Finished(NearResult<()>),
}
