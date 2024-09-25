pub mod sqlite_storage;

use near_base::{Deserialize, NearResult, Serialize};

pub trait ItemTrait: Serialize + Deserialize {
    fn id(&self) -> &str;
}

// trait StorageCloneTrait<T: ItemTrait + Send + Sync> {
//     fn clone_as_storage(&self) -> Box<dyn StorageTrait<T>>;
// }

#[async_trait::async_trait]
pub trait StorageTrait<T: ItemTrait + Send + Sync>: Send + Sync {

    async fn load(&self) -> NearResult<Vec<T>>;
    async fn load_with_prefix(&self, prefix: &str) -> NearResult<T>;
    async fn create_new(&self, data: &T) -> NearResult<()>;
    async fn update(&self, data: &T) -> NearResult<()>;
    async fn delete_with_prefix(&self, prefix: &str) -> NearResult<()>;

    async fn begin(&self) -> NearResult<Box<dyn StorageTransactionTrait<T>>>;
}

#[async_trait::async_trait]
pub trait StorageTransactionTrait<T: ItemTrait + Send + Sync>: StorageTrait<T> {
    async fn rollback(&mut self) -> NearResult<()>;
    async fn commit(&mut self) -> NearResult<()>;
}
