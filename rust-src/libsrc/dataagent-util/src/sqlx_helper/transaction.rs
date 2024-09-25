
use std::{sync::Arc, collections::{BTreeMap, btree_map::Entry}};
use async_std::sync::Mutex;

use near_base::{NearResult, NearError, ErrorCode, Sequence};

use crate::Helper;

use super::sqlx_sqlite::{SqlTransaction, SqlStateWrapper};

pub struct Transaction {
    // conn: Option<SqlConnectionWrapper>,
    transaction: Option<SqlTransaction>,
}

unsafe impl Send for Transaction {}
unsafe impl Sync for Transaction {}

impl Transaction {
    pub(crate) fn new(trans: SqlTransaction) -> NearResult<Transaction> {
        Ok(Self{
            transaction: Some(trans),
        })
    }

    pub async fn commit(self) -> NearResult<()> {
        if let Some(trans) = self.transaction {
            trans.commit().await
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "invalid param."))
        }
    }

    pub async fn rollback(self) -> NearResult<()> {
        if let Some(trans) = self.transaction {
            trans.rollback().await
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "invalid param."))
        }
    }
}

impl Transaction {
    pub async fn query_all<T>(&mut self, 
                              sql_id: &str) -> NearResult<Vec<T>>
    where T: protobuf::MessageFull {
        SqlStateWrapper::query_all(self.transaction.as_mut().unwrap(), sql_id)
            .await
    }

    pub async fn query_all_with_param<T>(&mut self, 
                                         sql_id: &str, 
                                         params: impl protobuf::MessageFull) -> NearResult<Vec<T>>
    where T: protobuf::MessageFull {
        SqlStateWrapper::query_all_with_param(self.transaction.as_mut().unwrap(), sql_id, params)
            .await
    }

    pub async fn execute(&mut self, sql_id: &str) -> NearResult<()> {
        SqlStateWrapper::execute(self.transaction.as_mut().unwrap(), sql_id)
            .await
    }

    pub async fn execute_with_param(&mut self, sql_id: &str, params: &impl protobuf::MessageFull) -> NearResult<()> {
        SqlStateWrapper::execute_with_param(self.transaction.as_mut().unwrap(), sql_id, params)
            .await
    }

}

struct ManagerImpl {
    sequence: Sequence,
    list: Mutex<BTreeMap<u32, Transaction>>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn get_instance() -> &'static Manager {
        static INSTANCE: once_cell::sync::OnceCell<Manager> = once_cell::sync::OnceCell::new();

        INSTANCE.get_or_init(|| {
            let r = Self(Arc::new(ManagerImpl {
                sequence: Sequence::random(),
                list: Mutex::new(BTreeMap::new()),
            }));

            r
        })
    }

    pub async fn begin_transaction(&self, db: &Helper) -> NearResult<u32> {
        let trans = db.begin_transaction().await?;
        let seq = self.0.sequence.generate().into_value();

        if let Entry::Vacant(empty) = self.0.list.lock().await.entry(seq) {
            empty.insert(trans);
        } else {
            unreachable!()
        }

        Ok(seq)
    }

    pub async fn commit(&self, id: u32) -> NearResult<()> {
        let trans = 
            self.0.list
                .lock().await
                .remove(&id)
                .ok_or_else(|| {
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found {id} transaction."))
                })?;

        trans.commit().await
    }

    pub async fn rollback(&self, id: u32) -> NearResult<()> {
        let trans = 
            self.0.list
                .lock().await
                .remove(&id)
                .ok_or_else(|| {
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found {id} transaction."))
                })?;

        trans.rollback().await
    }

}

unsafe impl Send for Manager {}
unsafe impl Sync for Manager {}

impl Manager {
    pub async fn query_all<T>(&self, id: u32, sql_id: &str) -> NearResult<Vec<T>>
    where T: protobuf::MessageFull {
        self.0.list
            .lock().await
            .get_mut(&id)
            .ok_or_else(|| {
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found {id} transaction."))
            })?
            .query_all(sql_id)
            .await
    }

    pub async fn query_all_with_param<T>(&self, id: u32, sql_id: &str, params: impl protobuf::MessageFull) -> NearResult<Vec<T>>
    where T: protobuf::MessageFull {
        self.0.list
            .lock().await
            .get_mut(&id)
            .ok_or_else(|| {
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found {id} transaction."))
            })?
            .query_all_with_param(sql_id, params)
            .await
    }

    pub async fn execute(&self, id: u32, sql_id: &str) -> NearResult<()> {
        self.0.list
            .lock().await
            .get_mut(&id)
            .ok_or_else(|| {
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found {id} transaction."))
            })?
            .execute(sql_id)
            .await
    }

    pub async fn execute_with_param(&self, id: u32, sql_id: &str, params: &impl protobuf::MessageFull) -> NearResult<()> {
        self.0.list
            .lock().await
            .get_mut(&id)
            .ok_or_else(|| {
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found {id} transaction."))
            })?
            .execute_with_param(sql_id, params)
            .await
    }
}

