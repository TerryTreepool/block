
use std::{sync::Arc, collections::BTreeMap, path::{PathBuf}, io::SeekFrom};

use async_std::{io::{WriteExt, prelude::SeekExt}, };
use near_base::{NearResult, NearError, ErrorCode, };
use near_core::{get_data_path, get_temp_path};
use near_util::Helper;
use serde_json::Number;

use crate::{Config, http_server::{HttpServer, HttpEventTrait, UrlRequest}, p::{FileArticlePtr, HttpResult, ToHttpResult, FILE_MANAGER}, nds_process::{NdsManager}};

struct AppImpl {
    config: Config,
    sqlx_helper: Arc<Helper>,
    nds_manager: NdsManager,
}

const GATEWAY_DATA_DB: &'static str = "gw_data.db";

const DEFAULT_STRING: &'static str = "default";

#[derive(Clone)]
pub struct App(Arc<AppImpl>);

impl App {
    pub async fn new(config: Config) -> NearResult<Self> {

        let sqlx_helper = App::init_sqlx().await?;

        let app = Self(Arc::new(AppImpl {
            config,
            sqlx_helper: Arc::new(sqlx_helper),
            nds_manager: NdsManager::new(),
        }));

        Ok(app)
    }

    async fn init_sqlx() -> NearResult<Helper> {
        let sqlx_helper = Helper::new(get_data_path().join(GATEWAY_DATA_DB).as_path(), Default::default())?;

        let file_article =
            String::from(r#"CREATE TABLE IF NOT EXISTS file_article (
                "file_id" varchar(100) PRIMARY KEY NOT NULL UNIQUE,
                "uid" varchar(100),
                "file_name" varchar(256),
                "file_size" INTEGER,
                "create_time" TIME,
                "flag" INTEGER);"#);
        sqlx_helper.add_sql("create_file_article", file_article)?;
        sqlx_helper.execute("create_file_article", Default::default()).await?;

        let insert_file_article =
            String::from(r#"insert into file_article(file_id, uid, file_name, file_size, create_time, flag)
                values(#file_id#, #uid#, #file_name#, #file_size#, strftime('%Y-%m-%d %H:%M:%S','now'), false);"#);
        sqlx_helper.add_sql("insert_file_article", insert_file_article)?;

        // let query_file_article =
        //     String::from(r#"select file_id, uid, file_name, file_size, create_time, flag from file_article;"#);
        let query_file_article =
            String::from(r#"select file_id, uid, file_name from file_article;"#);
        sqlx_helper.add_sql("query_file_article", query_file_article)?;

        sqlx_helper.query_all("query_file_article", BTreeMap::default())
                .await
                .iter()
                .for_each(| v | {
                    let _ = v.iter()
                            .for_each(| map | {
                                if let Some(file_id) = map.get("file_id") {
                                    let uid = map.get("uid").map(| d | d.clone()).unwrap_or(DEFAULT_STRING.to_string());
                                    let file_name = map.get("file_name").map(|d| d.clone() ).unwrap_or(DEFAULT_STRING.to_string());
                                    let file_size = map.get("file_size").map(| s | s.parse::<u64>().unwrap_or(0) ).unwrap_or(0);
                                    let flag = map.get("flag").map(| s | s.parse::<u8>().unwrap_or(0) ).unwrap_or(0);

                                    FILE_MANAGER.add_file(FileArticlePtr::with_info(file_id.clone(), uid, file_name, file_size, flag));
                                }
                            });
                });

        Ok(sqlx_helper)
    }

    pub fn config(&self) -> &Config {
        &self.0.config
    }

    pub async fn start(&self) -> NearResult<()> {
        HttpServer::new(self.clone())?
            .start(self.config().port)
            .await
    }
}

#[async_trait::async_trait]
impl HttpEventTrait for App {
    async fn post(&self, req: UrlRequest<'_>) -> NearResult<Box<dyn ToHttpResult>> {
        self.request_impl(req).await

    }

    async fn get(&self, req: UrlRequest<'_>) -> NearResult<Box<dyn ToHttpResult>> {
        self.request_impl(req).await
    }

}

// async fn post(&self, req: UrlRequest<'_>) -> NearResult<R>;


impl App {
    async fn request_impl(&self, req: UrlRequest<'_>) -> NearResult<Box<dyn ToHttpResult>> {
        match req.operator {
            "create_file_article" => self.create_file_article(req).await,
            "get_file_content" => self.get_file_content(req).await,
            "get_file_list" => self.get_file_list(req).await,
            "upload_file" => self.upload_file(req).await,
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN, format!("[{}] is unknown protocol", req.operator)))
            }
        }
    }

    async fn create_file_article(&self, req: UrlRequest<'_>) -> NearResult<Box<dyn ToHttpResult>> {
        let file_name = *req.params.get("file_name").ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "Not found [file_name] field"))?;
        let file_size = *req.params.get("file_size").ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "Not found [file_size] field"))?;

        let file_article =
            FileArticlePtr::new(req.uid.to_string(),
                                file_name.to_string(),
                                file_size.parse::<u64>().map_err(| _ | NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, format!("The file_size [{}] is invalid", file_size)))?
            )?;

        self.0.sqlx_helper.execute("insert_file_article", file_article.into_map()).await
            .map_err(| err | {
                println!("failed execute with err {}", err);
                err
            })?;

        let _ = FILE_MANAGER.add_file(file_article.clone());

        async_std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(get_temp_path().join(file_article.file_id()).as_path())
            .await
            .map_err(| err | {
                println!("failed create [{}] with err {}", file_article.file_id(), err);
                err
            })?;

        Ok(file_article.into_resp())
    }

    async fn upload_file(&self, req: UrlRequest<'_>) -> NearResult<Box<dyn ToHttpResult>> {
        let file_id = *req.params.get("file_id").ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "Not found [file_id] field"))?;

        let file_article = 
            FILE_MANAGER.get_file(file_id)
                        .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("[{}] not found it.", file_id) ))?;

        let mut file = 
            async_std::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(get_temp_path().join(file_article.file_id()).as_path())
                .await
                .map_err(| err | {
                    println!("failed open [{}] with err {}", file_article.file_id(), err);
                    err
                })?;

        let file_length = file.seek(SeekFrom::End(0)).await?;

        let writed_length = 
            file.write(req.body.as_slice())
                .await
                .map(| size | size as u64)
                .map_err(| err |{
                    println!("failed open [{}] with err {}", file_id, err);
                    err
                })?;

        if file_length + writed_length == file_article.file_size() {
            // finished
            self.0.nds_manager.add_nds_process(file_article.into_nds_file_article()?)?;
        }

        struct OkResp;

        impl<'a> ToHttpResult for OkResp {
            fn to_result(&self) -> HttpResult {
                HttpResult::default()
            }
        }

        Ok(Box::new(OkResp))
    }

    async fn get_file_list(&self, req: UrlRequest<'_>) -> NearResult<Box<dyn ToHttpResult>> {
        let index = {
            if let Some(&index) = req.params.get("index") {
                index.parse::<usize>()
                .map_err(| _ | {
                    NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, format!("[{}] is invalid int format", index))
                })
            } else {
                Ok(0usize)
            }
        }?;

        struct GetFileListResp {
            file_list: Vec<FileArticlePtr>,
            page_count: usize,
        }

        impl<'a> ToHttpResult for GetFileListResp {
            fn to_result(&self) -> HttpResult {
                let mut map = serde_json::Map::new();

                let mut array: Vec<serde_json::Value> = vec![];

                self.file_list
                    .iter()
                    .for_each(| f | {
                        let mut p = serde_json::Map::new();
                        p.insert("file_id".to_string(), serde_json::Value::String(f.file_id().clone()));
                        p.insert("file_name".to_string(), serde_json::Value::String(f.file_name().clone()));
                        p.insert("state".to_string(), serde_json::Value::Number(Number::from(f.state())));

                        array.push(p.into());
                    });
                // let array: serde_json::Value = array.into();

                map.insert("page_count".to_string(), serde_json::Value::Number(Number::from(self.page_count)));
                map.insert("file_array".to_string(), array.into());

                HttpResult::from((BTreeMap::new(), serde_json::Value::Object(map)))
            }
        }

        let (file_list, page_count) = FILE_MANAGER.filter_file(index);

        let r = Box::new(GetFileListResp { file_list, page_count });

        Ok(r)
    }

    async fn get_file_content(&self, req: UrlRequest<'_>) -> NearResult<Box<dyn ToHttpResult>> {
        let file_id = *req.params.get("file_id").ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "Not found [file_id] field"))?;

        let _ = FILE_MANAGER.get_file(file_id)
                    .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("[{}] not found it.", file_id) ))?;

        let path = get_temp_path().join(file_id);

        let _ =
        async_std::fs::OpenOptions::new()
            .read(true)
            .open(path.as_path())
            .await
            .map_err(| err | {
                println!("failed open [{}] with err {}", file_id, err);
                err
            })?;

        struct FileContentResp(PathBuf);

        impl<'a> ToHttpResult for FileContentResp {
            fn to_result(&self) -> HttpResult {
                HttpResult::from((BTreeMap::new(), self.0.clone()))
            }
        }

        Ok(Box::new(FileContentResp(path)))
    }
}
