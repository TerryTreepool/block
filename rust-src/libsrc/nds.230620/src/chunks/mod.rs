
mod manager;
mod view;
mod store;
mod source;
// mod track;

pub use manager::Manager;
pub use view::{ChunkView, };

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, )]
pub enum ChunkState {
    Unknown = 0,
    NotFound = 1, // 不存在
    Pending = 2,  // 准备中
    OnAir = 3,
    Ready = 4,  // 就绪
    Ignore = 5, // 被忽略
}

#[repr(u16)]
pub enum ChunkAccess {
    Read = 1,
    Write,
}
