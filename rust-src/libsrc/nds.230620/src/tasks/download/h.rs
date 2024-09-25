
use near_base::{NearResult, };

use crate::{tasks::{manager::TaskTrait, }, nds_protocol::PieceMessage};

#[async_trait::async_trait]
pub trait OnEventTrait: Send + Sync {
    async fn on_piece_data(&self, data: &PieceMessage) -> NearResult<()>;
}

pub trait DownloadTaskTrait : TaskTrait + OnEventTrait {
    fn clone_as_downloadtask(&self) -> Box<dyn DownloadTaskTrait>;
}
