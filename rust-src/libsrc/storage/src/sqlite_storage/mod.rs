
mod storage;
mod sqlmap;

pub use storage::SqliteStorage;

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StorageType {
    Init = 0,
    QueryOne = 1,
    QueryAll = 2,
    Create = 3,
    Update = 4,
    Delete = 5,
}
