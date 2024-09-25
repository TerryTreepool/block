
mod manager;
mod tasks;

pub use manager::Manager;

#[derive(Clone)]
pub enum ChunkEncodeDesc {
    Stream(Option<u32>, Option<u32>, Option<i32>), 
    Raptor(Option<u32>, Option<u32>, Option<i32>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, )]
pub enum ChunkState {
    Unknown = 0,
    NotFound = 1, // 不存在
    Pending = 2,  // 准备中
    OnAir = 3,
    Ready = 4,  // 就绪
    Ignore = 5, // 被忽略
}

