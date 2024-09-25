use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use async_std::sync::Mutex as AsyncMutex;

use log::{error, info, trace};
use near_base::{ErrorCode, NearError, NearResult};

use dataagent_util::{Helper, Transaction};
use protos::profile::Data;

use crate::{
    sqlite_storage::{sqlmap::SqlmapBuild, StorageType},
    ItemTrait, StorageTrait, StorageTransactionTrait,
};

type HelperRef = Arc<Helper>;

pub struct SqliteStorage {
    helper: HelperRef,
    locker: Mutex<()>,
}

impl SqliteStorage {
    pub fn new(db: &Path) -> NearResult<Self> {
        Ok(Self {
            helper: HelperRef::new(Helper::new(db, None)?),
            locker: Mutex::new(()),
        })
    }

    pub async fn add_storage<T: ItemTrait + Send + Sync + Clone + 'static>(
        &self,
        name: &str,
    ) -> NearResult<Box<dyn StorageTrait<T>>> {
        trace!("add storage: {name}");

        let mut sqlmaps = vec![];

        for mode in vec![
            StorageType::Init,
            StorageType::QueryOne,
            StorageType::QueryAll,
            StorageType::Create,
            StorageType::Update,
            StorageType::Delete,
        ] {
            sqlmaps.push(SqlmapBuild { name, mode }.build());
        }

        let _locker = self.locker.lock().unwrap();

        for sqlmap in sqlmaps {
            let (name, input, output, sql) = sqlmap;
            let _ = self.helper.add_sql(
                name,
                input.as_ref().map(|v| v.as_str()),
                output.as_ref().map(|v| v.as_str()),
                sql,
            );
        }

        let init_sqlmap = 
            SqlmapBuild {
                name,
                mode: StorageType::Init,
            }
            .build_sql_key();

        let _ = self.helper.execute(init_sqlmap.as_str()).await?;

        let r = SqliteSubStorage::<T>::new(self.helper.clone(), name);

        Ok(Box::new(r) as Box<dyn StorageTrait<T>>)
    }
}

struct SqliteSubStorageImpl<T> {
    helper: HelperRef,
    name: String,
    _marker: std::marker::PhantomData<T>,
}

#[derive(Clone)]
struct SqliteSubStorage<T>(Arc<SqliteSubStorageImpl<T>>);

impl<T> SqliteSubStorage<T> {
    pub fn new(helper: HelperRef, name: &str) -> Self {
        Self(Arc::new(SqliteSubStorageImpl {
            helper,
            name: name.to_owned(),
            _marker: Default::default(),
        }))
    }
}

#[allow(non_snake_case)]
mod EncodeAndDecode {
    use crate::ItemTrait;
    use log::error;
    use near_base::NearResult;

    pub fn try_encode(data: &impl ItemTrait) -> NearResult<protos::profile::Data> {
        Ok(protos::profile::Data {
            key: data.id().to_owned(),
            value: {
                let mut buff = vec![0u8; data.raw_capacity()];
                let _ = data.serialize(&mut buff).map_err(|e| {
                    error!("failed serialize [{}] with err: {e}", data.id());
                    e
                })?;
                buff
            },
            ..Default::default()
        })
    }

    pub fn try_decode<T: ItemTrait>(data: &protos::profile::Data) -> NearResult<T> {
        T::deserialize(data.value()).map(|(v, _)| v).map_err(|e| {
            error!("failed deserialize [{}] with err: {e}", data.key());
            e
        })
    }
}

#[async_trait::async_trait]
impl<T: ItemTrait + Send + Sync + Clone + 'static> StorageTrait<T> for SqliteSubStorage<T> {

    async fn load(&self) -> NearResult<Vec<T>> {
        let sqlmap = SqlmapBuild {
            name: &self.0.name,
            mode: StorageType::QueryAll,
        }
        .build_sql_key();

        let dataes = self
            .0.helper
            .query_all::<Data>(&sqlmap)
            .await
            .map_err(|e| {
                error!("failed load data with err: {e}");
                e
            })?;

        let mut ret = vec![];
        for data in dataes {
            if let Ok(v) = EncodeAndDecode::try_decode(&data) {
                ret.push(v);
            }
        }

        Ok(ret)
    }

    async fn load_with_prefix(&self, prefix: &str) -> NearResult<T> {
        let sqlmap = SqlmapBuild {
            name: &self.0.name,
            mode: StorageType::QueryOne,
        }
        .build_sql_key();

        let data = self
            .0.helper
            .query_all_with_param::<Data>(
                &sqlmap,
                Data {
                    key: prefix.to_owned(),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| {
                error!("failed load [{prefix}] data with err: {e}");
                e
            })?
            .get(0)
            .map(|data| EncodeAndDecode::try_decode(data))
            .ok_or_else(|| {
                let error_string = format!("Not found [{prefix}] data");
                info!("{error_string}");
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
            })?
            .map_err(|e| {
                error!("failed deserialize [{prefix}]'s key with err: {e}");
                e
            })?;

        Ok(data)
    }

    async fn create_new(&self, data: &T) -> NearResult<()> {
        let sqlmap = SqlmapBuild {
            name: &self.0.name,
            mode: StorageType::Create,
        }
        .build_sql_key();

        self.0.helper
            .execute_with_param(&sqlmap, &EncodeAndDecode::try_encode(data)?)
            .await
            .map_err(|e| {
                error!("failed update [{}] with err: {e}", data.id());
                e
            })
    }

    async fn update(&self, data: &T) -> NearResult<()> {
        let sqlmap = SqlmapBuild {
            name: &self.0.name,
            mode: StorageType::Update,
        }
        .build_sql_key();

        self.0.helper
            .execute_with_param(&sqlmap, &EncodeAndDecode::try_encode(data)?)
            .await
            .map_err(|e| {
                error!("failed update [{}] with err: {e}", data.id());
                e
            })
    }

    async fn delete_with_prefix(&self, perfix: &str) -> NearResult<()> {
        let sqlmap = SqlmapBuild {
            name: &self.0.name,
            mode: StorageType::Delete,
        }
        .build_sql_key();

        self.0.helper
            .execute_with_param(
                &sqlmap,
                &Data {
                    key: perfix.to_owned(),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| {
                error!("failed delete [{perfix}] with err: {e}");
                e
            })
    }

    async fn begin(&self) -> NearResult<Box<dyn StorageTransactionTrait<T>>> {
        let transaction = 
            self.0.helper
                .begin_transaction()
                .await?;

        Ok(StorageTransactionImpl::new(self.clone(), transaction))
    }
}

struct StorageTransactionImpl<T> {
    storage: SqliteSubStorage<T>,
    transaction: AsyncMutex<Option<Transaction>>,
}

impl<T: ItemTrait + Send + Sync + Clone + 'static> StorageTransactionImpl<T> {
    pub fn new(storage: SqliteSubStorage<T>, transaction: Transaction) -> Box<dyn StorageTransactionTrait<T>> {
        Box::new(Self {
            storage,
            transaction: AsyncMutex::new(Some(transaction)),
        }) as Box<dyn StorageTransactionTrait<T>>
    }
}

// impl<T> std::ops::Drop for StorageTransactionImpl<T> {
//     fn drop(&mut self) {
//         let transaction = {
//             let transaction = &mut *self.transaction.lock().await;
//             std::mem::replace(transaction, None)
//         };

//         if let Some(transaction) = transaction {
//             async_std::task::spawn(async move {
//                 let _ = transaction.rollback().await;
//             });
//         }
//     }
// }

#[async_trait::async_trait]
impl<T: ItemTrait + Send + Sync + Clone + 'static> StorageTransactionTrait<T> for StorageTransactionImpl<T> {

    async fn rollback(&mut self) -> NearResult<()> {
        let transaction = {
            let transaction = &mut *self.transaction.lock().await;
            std::mem::replace(transaction, None)
        };

        if let Some(transaction) = transaction {
            transaction.rollback().await
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, "invalid transaction."))
        }
    }

    async fn commit(&mut self) -> NearResult<()> {
        let transaction = {
            let transaction = &mut *self.transaction.lock().await;
            std::mem::replace(transaction, None)
        };

        if let Some(transaction) = transaction {
            transaction.commit().await
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, "invalid transaction."))
        }
    }
}

#[async_trait::async_trait]
impl<T: ItemTrait + Send + Sync + Clone + 'static> StorageTrait<T> for StorageTransactionImpl<T> {

    async fn load(&self) -> NearResult<Vec<T>> {

        let sqlmap = SqlmapBuild {
            name: &self.storage.0.name,
            mode: StorageType::QueryAll,
        }
        .build_sql_key();

        let dataes = 
            if let Some(transactin) = &mut *self.transaction.lock().await {
                transactin.query_all::<Data>(&sqlmap)
                    .await
                    .map_err(|e| {
                        error!("failed load data with err: {e}");
                        e
                    })
            } else {
                Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, "invalid transaction."))
            }?;

        let mut ret = vec![];
        for data in dataes {
            if let Ok(v) = EncodeAndDecode::try_decode(&data) {
                ret.push(v);
            }
        }

        Ok(ret)
    }

    async fn load_with_prefix(&self, prefix: &str) -> NearResult<T> {
        let sqlmap = SqlmapBuild {
            name: &self.storage.0.name,
            mode: StorageType::QueryOne,
        }
        .build_sql_key();

        let dataes = 
            if let Some(transactin) = &mut *self.transaction.lock().await {
                transactin.query_all_with_param::<Data>(
                        &sqlmap,
                        Data {
                            key: prefix.to_owned(),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| {
                        error!("failed load [{prefix}] data with err: {e}");
                        e
                    })
            } else {
                Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, "invalid transaction."))
            }?;

            dataes.get(0)
                .map(|data| EncodeAndDecode::try_decode(data))
                .ok_or_else(|| {
                    let error_string = format!("Not found [{prefix}] data");
                    info!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
                })?
                .map_err(|e| {
                    error!("failed deserialize [{prefix}]'s key with err: {e}");
                    e
                })
    }

    async fn create_new(&self, data: &T) -> NearResult<()> {
        let sqlmap = SqlmapBuild {
            name: &self.storage.0.name,
            mode: StorageType::Create,
        }
        .build_sql_key();

        if let Some(transactin) = &mut *self.transaction.lock().await {
            transactin.execute_with_param(&sqlmap, &EncodeAndDecode::try_encode(data)?)
                .await
                .map_err(|e| {
                    error!("failed update [{}] with err: {e}", data.id());
                    e
                })
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, "invalid transaction."))
        }
    }

    async fn update(&self, data: &T) -> NearResult<()> {
        let sqlmap = SqlmapBuild {
            name: &self.storage.0.name,
            mode: StorageType::Update,
        }
        .build_sql_key();

        if let Some(transactin) = &mut *self.transaction.lock().await {
            transactin.execute_with_param(&sqlmap, &EncodeAndDecode::try_encode(data)?)
                .await
                .map_err(|e| {
                    error!("failed update [{}] with err: {e}", data.id());
                    e
                })
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, "invalid transaction."))
        }

    }

    async fn delete_with_prefix(&self, perfix: &str) -> NearResult<()> {
        let sqlmap = SqlmapBuild {
            name: &self.storage.0.name,
            mode: StorageType::Delete,
        }
        .build_sql_key();

        if let Some(transactin) = &mut *self.transaction.lock().await {
            transactin.execute_with_param(
                &sqlmap,
                &Data {
                    key: perfix.to_owned(),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| {
                error!("failed delete [{perfix}] with err: {e}");
                e
            })
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, "invalid transaction."))
        }
    }

    async fn begin(&self) -> NearResult<Box<dyn StorageTransactionTrait<T>>> {
        Err(NearError::new(ErrorCode::NEAR_ERROR_ALREADY_EXIST, "exist."))
    }

}

#[allow(unused)]
mod test {
    use std::path::PathBuf;

    use crate::ItemTrait;

    use super::SqliteStorage;

    #[test]
    fn test_storage() {

        impl ItemTrait for u32 {
            fn id(&self) -> &str {
                "100"
            }
        }

        async_std::task::block_on(async move {
            let helper = SqliteStorage::new(PathBuf::new().join("C:\\1.db").as_path()).unwrap();
            let tb = helper.add_storage("test").await.unwrap();

            let mut tr = tb.begin().await.unwrap();

            // tb.create_new(&100u32).await.unwrap();

            tr.create_new(&100u32).await.unwrap();

            tr.commit().await.unwrap();

        })
    }
}
