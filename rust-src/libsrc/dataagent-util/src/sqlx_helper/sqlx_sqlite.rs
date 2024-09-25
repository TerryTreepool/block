
use std::{time::Duration, str::FromStr};
use log::error;

use near_base::{NearResult, NearError, ErrorCode, ToNearError};
use sqlx::{pool::PoolConnection, 
           Any, 
           Error as SqlxError, 
           Executor, Transaction, 
    };

use crate::Helper;

#[async_trait::async_trait]
pub trait SqlxEventTrait {
    async fn query_one(&mut self, params: SqlArguments<'_>) -> NearResult<SqlRowObject>;
    async fn query_all(&mut self, params: SqlArguments<'_>) -> NearResult<Vec<SqlRowObject>>;
    async fn execute(&mut self, params: SqlArguments<'_>) -> NearResult<SqlResult>;
}

struct DBError<'error> {
    err: SqlxError,
    fromer: &'error str
}

impl<'error> From<(SqlxError, &'error str /* fromer */)> for DBError<'error> {
    fn from(cx: (SqlxError, &'error str /* fromer */)) -> Self {
        let (err, fromer) = cx;

        Self {
            err, fromer
        }
           
    }
}

impl<'error> ToNearError for DBError<'error> {
    fn to_near_error(self) -> NearError {
        match &self.err {
            sqlx::Error::Database(e) if e.code().is_some() => {
                let code = e.code().unwrap();
                if code.parse::<i32>().unwrap_or(0) == 1555 {
                    NearError::new(ErrorCode::NEAR_ERROR_ALREADY_EXIST, "existed")
                } else {
                    let error_message = format!("failed to sqlite with err: {} at {}", e.to_string(), self.fromer);
                    error!("{}", error_message);
                    NearError::new(ErrorCode::NEAR_ERROR_3RD, error_message)
                }
            }
            sqlx::Error::RowNotFound => {
                let error_message = format!("not found with err: {} at {}", self.err.to_string(), self.fromer);
                error!("{}", error_message);
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_message)
            }
            _ => {
                let error_message = format!("failed to sqlite with err: {} at {}", self.err.to_string(), self.fromer);
                error!("{}", error_message);
                NearError::new(ErrorCode::NEAR_ERROR_3RD, error_message)
            }
        }
    }
}

pub struct Config {
    min_connections: u32,
    max_connections: u32,
    connect_timeout: Duration,
    idle_timeout: Duration,
    busy_timeout: Duration,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            min_connections: 0, 
            max_connections: 4, 
            connect_timeout: Duration::from_secs(300), 
            idle_timeout: Duration::from_secs(300),
            busy_timeout: Duration::from_secs(300),
        }
    }
}

impl Config {
    #[allow(unused)]
    pub fn set_min_connections(mut self, min_connections: u32) -> Self {
        self.min_connections = min_connections;
        self
    }

    #[allow(unused)]
    pub fn set_max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }

    #[allow(unused)]
    pub fn set_connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = connect_timeout;
        self
    }

    #[allow(unused)]
    pub fn set_idle_timeout(mut self, idle_timeout: Duration) -> Self {
        self.idle_timeout = idle_timeout;
        self
    }

    #[allow(unused)]
    pub fn set_busy_timeout(mut self, busy_timeout: Duration) -> Self {
        self.busy_timeout = busy_timeout;
        self
    }
}

pub struct Pool {
    pool: sqlx::AnyPool,

}

impl Pool {
    pub fn open(uri: &str, config: Config) -> NearResult<Self> {
        let pool_options = 
            sqlx::any::AnyPoolOptions::new()
                .max_connections(config.max_connections)
                // .connect_timeout(config.connect_timeout)
                .acquire_timeout(config.connect_timeout)
                .min_connections(config.min_connections)
                .idle_timeout(config.idle_timeout);

        let pool = 
            match sqlx::any::AnyKind::from_str(uri)
                .map_err(| err | {
                    DBError::from((err, file!())).to_near_error()
                })? {
            sqlx::any::AnyKind::Sqlite => {
                let options = 
                    sqlx::sqlite::SqliteConnectOptions::from_str(uri)
                        .map_err(| err | {
                            DBError::from((err, file!())).to_near_error()
                        })?
                        .busy_timeout(config.busy_timeout)
                        .create_if_missing(true);

                #[cfg(target_os = "ios")]
                {
                    options = options.serialized(true);
                }

                pool_options.connect_lazy_with(sqlx::any::AnyConnectOptions::from(options))
            }
            // _ => {
            //     pool_options.connect_lazy(uri)
            //         .map_err(| err | {
            //             DBError::from((err, file!())).to_near_error()
            //         })?
            // }
        };

        Ok(Self { pool })       
    }

    pub async fn get_conn(&self) -> NearResult<SqlConnection> {
        self.pool
            .acquire()
            .await
            .map(| conn | {
                SqlConnection::from(conn)
            })
            .map_err(| err | {
                DBError::from((err, file!())).to_near_error()
            })
    }

}

pub struct SqlConnection {
    conn: PoolConnection<Any>,
}

impl From<PoolConnection<Any>> for SqlConnection {
    fn from(conn: PoolConnection<Any>) -> Self {
        Self{ conn }
    }
}

pub(crate) type SqlArguments<'a> = sqlx::query::Query<'a, sqlx::Any, <sqlx::Any as sqlx::database::HasArguments<'a>>::Arguments>;

pub struct SqlArgumentsStruct<'a> {
    arguments: SqlArguments<'a>,
}

impl<'a> From<&'a str> for SqlArgumentsStruct<'a> {
    fn from(sql: &'a str) -> Self {
        Self {
            arguments: sqlx::query::<sqlx::Any>(sql),
        }
    }
}

impl<'a> SqlArgumentsStruct<'a> {

    pub fn take(self) -> SqlArguments<'a> {
        self.arguments
    }

}

pub type SqlRowObject = <sqlx::Any as sqlx::Database>::Row;
pub type SqlResult = <sqlx::Any as sqlx::Database>::QueryResult;

#[async_trait::async_trait]
impl SqlxEventTrait for SqlConnection {
    async fn query_one(&mut self, params: SqlArguments<'_>) -> NearResult<SqlRowObject> {
        self.conn.fetch_one(params)
            .await
            .map_err(| err | {
                DBError::from((err, file!())).to_near_error()
            })
    }

    async fn query_all(&mut self, params: SqlArguments<'_>) -> NearResult<Vec<SqlRowObject>> {
        self.conn.fetch_all(params)
            .await
            .map_err(| err | {
                DBError::from((err, file!())).to_near_error()
            })
    }

    async fn execute(&mut self, params: SqlArguments<'_>) -> NearResult<SqlResult> {
        self.conn
            .execute(params)
            .await
            .map_err(| err | {
                DBError::from((err, file!())).to_near_error()
            })
    }

}

// impl SqlConnection {
//     #[allow(unused)]
//     pub async fn begin(&mut self) -> NearResult<SqlTransaction> {
//         SqlTransaction::begin(self).await
//     }
// }
pub(crate) struct SqlStateWrapper;

impl SqlStateWrapper {
    pub async fn query_all<E, T>(execute: &mut E, 
                                 sql_id: &str) -> NearResult<Vec<T>>
    where   E: SqlxEventTrait,
            T: protobuf::MessageFull {
        let sql_struct = super::p::Manager::get_instance().get_sql(sql_id)?;

        let sql_query = SqlArgumentsStruct::from(sql_struct.sql()).take();

        if sql_struct.sql_params().len() > 0 {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_DATAAGNT_NEED_BIND_PARAMS, "Need bind params, you can use query_all_with_param()"));
        }

        let mut all = vec![];

        let row_array = execute.query_all(sql_query).await?;

        for row in row_array.iter() {
            let mut v: Box<T> = Helper::get_variable(row)?;
            let mut default_v = T::default();
            std::mem::swap(v.as_mut(), &mut default_v);
            all.push(default_v);
        }

        Ok(all)
    }

    pub async fn query_all_with_param<E, T>(execute: &mut E, 
                                            sql_id: &str, 
                                            params: impl protobuf::MessageFull) -> NearResult<Vec<T>>
    where   E: SqlxEventTrait,
            T: protobuf::MessageFull {
        let sql_struct = super::p::Manager::get_instance().get_sql(sql_id)?;

        let mut sql_query = SqlArgumentsStruct::from(sql_struct.sql()).take();

        for item in sql_struct.sql_params() {
            sql_query = Helper::bind_variable(sql_query, item, &params)?;
        }    

        let mut all = vec![];

        let row_array = execute.query_all(sql_query).await?;

        for row in row_array.iter() {
            let mut v: Box<T> = Helper::get_variable(row)?;
            let mut default_v = T::default();
            std::mem::swap(v.as_mut(), &mut default_v);
            all.push(default_v);
        }

        Ok(all)
    }

    pub async fn execute<E>(execute: &mut E, sql_id: &str) -> NearResult<()> 
    where E: SqlxEventTrait {
        let sql_struct = super::p::Manager::get_instance().get_sql(sql_id)?;

        let sql_query = SqlArgumentsStruct::from(sql_struct.sql()).take();

        if sql_struct.sql_params().len() > 0 {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_DATAAGNT_NEED_BIND_PARAMS, "Need bind params, you can use query_all_with_param()"));
        }

        execute.execute(sql_query).await?;

        Ok(())
    }

    pub async fn execute_with_param<E>(execute: &mut E, sql_id: &str, params: &impl protobuf::MessageFull) -> NearResult<()>
    where   E: SqlxEventTrait {
        let sql_struct = super::p::Manager::get_instance().get_sql(sql_id)?;

        let mut sql_query = SqlArgumentsStruct::from(sql_struct.sql()).take();
        for item in sql_struct.sql_params() {
            sql_query = Helper::bind_variable(sql_query, item, params)?;
        }

        execute.execute(sql_query).await?;

        Ok(())
    }
}

pub struct SqlTransaction {
    trans: Transaction<'static, Any>,
}

impl std::ops::Deref for SqlTransaction {
    type Target = Transaction<'static, Any>;

    fn deref(&self) -> &Self::Target {
        &self.trans
    }
}

impl std::ops::DerefMut for SqlTransaction {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.trans
    }
}

impl SqlTransaction {
    // #[allow(unused)]
    // pub async fn begin(conn: &mut SqlConnection) -> NearResult<SqlTransaction> {
    //     let trans = conn.begin().await?
    //     // let this: &mut SqlConnection = unsafe { std::mem::transmute(conn) };

    //     // let trans = 
    //     //     this.conn.begin()
    //     //         .await
    //     //         .map_err(| err | {
    //     //             DBError::from((err, file!())).to_near_error()
    //     //         })?;

    //     // let r = SqlTransaction {
    //     //     trans: trans,
    //     // };

    //     Ok(r)
    // }

    #[allow(unused)]
    pub async fn begin(pool: &Pool) -> NearResult<SqlTransaction> {
        let trans = 
            pool.pool.begin().await
                .map_err(| err | {
                    DBError::from((err, file!())).to_near_error()
                })?;

        let r = SqlTransaction {
            trans: trans,
        };

        Ok(r)
    }
    #[allow(unused)]
    pub async fn rollback(mut self) -> NearResult<()> {
        self.trans
            .rollback()
            .await
            .map_err(| err | {
                DBError::from((err, file!())).to_near_error()
            })
    }

    #[allow(unused)]
    pub async fn commit(mut self) -> NearResult<()> {
        self.trans
            .commit()
            .await
            .map_err(| err | {
                DBError::from((err, file!())).to_near_error()
            })
    }

}

#[async_trait::async_trait]
impl SqlxEventTrait for SqlTransaction{
    async fn query_one(&mut self, params: SqlArguments<'_>) -> NearResult<SqlRowObject> {
        self.trans.fetch_one(params)
            .await
            .map_err(| err | {
                DBError::from((err, file!())).to_near_error()
            })
    }

    async fn query_all(&mut self, params: SqlArguments<'_>) -> NearResult<Vec<SqlRowObject>> {
        self.trans.fetch_all(params)
            .await
            .map_err(| err | {
                DBError::from((err, file!())).to_near_error()
            })
    }

    async fn execute(&mut self, params: SqlArguments<'_>) -> NearResult<SqlResult> {
        self.trans
            .execute(params)
            .await
            .map_err(| err | {
                DBError::from((err, file!())).to_near_error()
            })
    }
}

mod test {

    #[test]
    fn test_pool() {
        use sqlx::Row;
        use crate::sqlx_helper::sqlx_sqlite::SqlTransaction;
        use crate::sqlx_helper::sqlx_sqlite::{SqlxEventTrait, SqlArgumentsStruct};
        use crate::sqlx_helper::sqlx_sqlite::Config;
        use crate::sqlx_helper::sqlx_sqlite::Pool;

        async_std::task::block_on(async {
            let data = near_core::get_data_path().join("a.db");
            let pool = Pool::open(format!("sqlite://{}", data.to_str().unwrap()).as_str(), Config::default()).unwrap();

            let mut sqlx_conn = pool.get_conn().await.unwrap();
            let create_table = r#"CREATE TABLE IF NOT EXISTS desc_extra (
                "obj_id" char(45) PRIMARY KEY NOT NULL UNIQUE,
                "rent_arrears" INTEGER,
                "rent_arrears_count" INTEGER,
                "rent_value" INTEGER,
                "coin_id" INTEGER,
                "data_len" INTEGER,
                "other_charge_balance" INTEGER);"#;
            let r = sqlx_conn.execute(SqlArgumentsStruct::from(create_table).take()).await.unwrap();
            println!("{:#?}", r);

            let mut trans = SqlTransaction::begin(&pool).await.unwrap();
            let insert = r#"insert into desc_extra (obj_id,
                rent_arrears,
                rent_arrears_count,
                rent_value,
                coin_id,
                data_len,
                other_charge_balance) values (
                "test", 1, 1, 2, 3, 4, 5)"#;
            trans.execute(SqlArgumentsStruct::from(insert).take()).await.unwrap();

            // let s1: SqlArguments<'_> = SqlArgumentsStruct::from("select * from desc_extra where obj_id = ?").take();
            // let s1 = s1.bind("test1");
            let row = trans.query_one(SqlArgumentsStruct::from("select * from desc_extra where obj_id = ?").take().bind("test")).await.unwrap();
            let id: String = row.get("obj_id");
            println!("obj_id: {}", id );

            let r = trans.commit().await;
            println!("{:#?}", r);
            // let query = sqlx::query("select * from desc_extra where obj_id = ?").bind("test");
            // let row = sqlx_conn.query_all(query).await.unwrap();
            // sqlx_conn.execute_sql(sql_query(insert)).await.unwrap();
            // sqlx_conn.rollback_transaction().await.unwrap();

            // let mut sqlx_conn = pool.get_conn().await.unwrap();
            // let query = sqlx::query("select * from desc_extra where obj_id = ?").bind("test");
            // let row = sqlx_conn.query_all(query).await.unwrap();
            // assert_eq!(row.len(), 0);

            // let mut sqlx_conn = pool.get_conn().await.unwrap();
            // sqlx_conn.begin_transaction().await.unwrap();
            // let insert = r#"insert into desc_extra (obj_id,
            // rent_arrears,
            // rent_arrears_count,
            // rent_value,
            // coin_id,
            // data_len,
            // other_charge_balance) values (
            // "test", 1, 1, 2, 3, 4, 5)"#;
            // sqlx_conn.execute_sql(sql_query(insert)).await.unwrap();
            // sqlx_conn.commit_transaction().await.unwrap();

            // let query = sqlx::query("select * from desc_extra where obj_id = ?").bind("test");
            // let row = sqlx_conn.query_one(query).await.unwrap();
            // let id: String = row.get("obj_id");
            // assert_eq!(id, "test".to_owned());
            // let coin_id: i32 = row.get("coin_id");
            // assert_eq!(coin_id, 3);

            // let row = sqlx_conn.query_one(sqlx::query("select * from desc_extra where obj_id = ?").bind("test")).await.unwrap();
            // let id: String = row.get("obj_id");
            // assert_eq!(id, "test".to_owned());
            // let coin_id: i32 = row.get("coin_id");
            // assert_eq!(coin_id, 3);
        })
    }

}
