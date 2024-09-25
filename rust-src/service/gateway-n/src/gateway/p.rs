
use std::{collections::{BTreeMap, }, sync::{RwLock, Arc, atomic::{AtomicU8, Ordering}}, };

use near_base::{NearResult, Serialize, Timestamp, now, NearError, ErrorCode, hash_data, };
use near_core::{path_utils::get_temp_path};
use once_cell::sync::OnceCell;
use tide::{Response, StatusCode, };

use super::nds_process::NdsFileArticle;


pub enum HttpResultType {
    Memory(Vec<u8>),
    File(std::path::PathBuf),
}

pub struct HttpResult {
    pub header: BTreeMap<String, String>,
    pub body: HttpResultType,

}

impl std::default::Default for HttpResult {
    fn default() -> Self {
        Self {
            header: BTreeMap::new(),
            body: HttpResultType::Memory(vec![]),
        }
    }
}

impl From<(BTreeMap<String, String>, serde_json::Value)> for HttpResult {
    fn from(cx: (BTreeMap<String, String>, serde_json::Value)) -> Self {
        let (header, body) = cx;

        Self {
            header, 
            body: HttpResultType::Memory(body.to_string().as_bytes().to_vec()),
        }
    }
}

impl From<(BTreeMap<String, String>, std::path::PathBuf)> for HttpResult {
    fn from(cx: (BTreeMap<String, String>, std::path::PathBuf)) -> Self {
        let (header, path) = cx;

        Self {
            header, 
            body: HttpResultType::File(path),
        }
    }
}

impl HttpResult {
    pub async fn into_response(self) -> Response {
        let mut resp = Response::new(StatusCode::Ok);

        for (k, v) in self.header {
            resp.append_header(k.as_str(), v);
        }

        match self.body {
            HttpResultType::Memory(data) => {
                resp.set_content_type("application/json");
                resp.set_body(data);
            }
            HttpResultType::File(path) => {
                resp.set_content_type("application/text");
                resp.set_body(tide::Body::from_file(path).await.unwrap())
            }
        }

        resp
    }
}

pub trait ToHttpResult: Send + Sync {
    fn to_result(&self) -> HttpResult;
}

pub enum FileStateImpl {
    Unknown,
    Prepair,
    Finished,
    NdsSyncing,
    NdsSynced,
}

impl FileStateImpl {
    pub fn into_u8(&self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::Prepair => 1,
            Self::Finished => 2,
            Self::NdsSyncing => 3,
            Self::NdsSynced => 4,
        }
    }
}

impl std::fmt::Display for FileStateImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.into_u8())
    }
}

impl From<u8> for FileStateImpl {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::Prepair,
            2 => Self::Finished,
            3 => Self::NdsSyncing,
            4 => Self::NdsSynced,
            _ => Self::Unknown,
        }
    }
}

// #[derive(Debug)]
struct FileArticle {
    uid: String,
    file_id: String,
    file_name: String,
    file_size: u64,
    #[allow(unused)]
    begin_time: Timestamp,
    state: AtomicU8, 
}

impl std::fmt::Debug for FileArticle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
               "uuid = {}, file_id = {}, file_name = {}, file_size = {}", 
               self.uid, self.file_id, self.file_name, self.file_size)
    }
}

#[derive(Clone)]
pub struct FileArticlePtr(Arc<FileArticle>);

impl std::fmt::Debug for FileArticlePtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FileArticlePtr {
    pub fn finished(&self) -> FileStateImpl {
        match self.0.state.compare_exchange(FileStateImpl::Prepair.into_u8(), 
                                            FileStateImpl::Finished.into_u8(), 
                                            Ordering::SeqCst, 
                                            Ordering::SeqCst) {
            Ok(succ) => {
                FileStateImpl::from(succ)
            }
            Err(fail) => {
                FileStateImpl::from(fail)
            }
        }
    }
}

pub struct FileArticleManager {
    files_vect: RwLock<Vec<FileArticlePtr>>,
    files: RwLock<BTreeMap<String, FileArticlePtr>>,
}

impl FileArticleManager {
    fn new() -> Self {
        Self {
            files_vect: RwLock::new(vec![]),
            files: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn get_instance() -> &'static Self {
        static INSTACNE: OnceCell<FileArticleManager> = OnceCell::new();
        INSTACNE.get_or_init(||{
            Self::new()
        })
    }

    pub fn add_file(&self, file: FileArticlePtr) {
        self.files.write().unwrap().insert(file.file_id().clone(), file.clone());
        self.files_vect.write().unwrap().push(file.clone());
    }

    pub fn get_file(&self, file_id: &str) -> Option<FileArticlePtr> {
        self.files.read().unwrap()
            .get(file_id)
            .map(| file | file.clone() )
    }

    pub fn filter_file(&self, index: usize) -> (Vec<FileArticlePtr>, usize /* page_count */) {
        let mut count = Self::page_max_count() as i32;

        {
            let files_vect = &*self.files_vect.read().unwrap();

            let r = 
            files_vect
                .iter()
                .skip(index * Self::page_max_count())
                .filter(| _ | {
                    count -= 1;
                    count >= 0
                })
                .map(| v | v.clone())
                .collect::<Vec<FileArticlePtr>>();

            let page_count = if files_vect.len() % Self::page_max_count() == 0 { files_vect.len() / Self::page_max_count() } else { files_vect.len() / Self::page_max_count() + 1 };

            (r, page_count)
        }
    }

    pub fn page_max_count() -> usize {
        50
    }

}

struct FileArticleResp {
    file_id: String,
}

impl FileArticleResp {
    fn new(file_id: String) -> Self {
        Self { file_id }
    }
}

impl<'a> ToHttpResult for FileArticleResp {
    fn to_result(&self) -> HttpResult {
        let mut j = serde_json::Map::new();
        j.insert("file_id".to_string(), serde_json::Value::String(self.file_id.clone()));
        let j: serde_json::Value = j.into();

        HttpResult::from((BTreeMap::new(), j))
    }
}

impl FileArticlePtr {
    pub fn new(uid: String, file_name: String, file_size: u64) -> NearResult<Self> {
        let now = now();

        let len = uid.raw_capacity() + file_name.raw_capacity() + file_size.raw_capacity() + now.raw_capacity();
        let file_id = {
            let mut buf = vec![0u8; len];
            let end = uid.serialize(buf.as_mut_slice())?;
            let end = file_name.serialize(end)?;
            let end = file_size.serialize(end)?;
            let _end = now.serialize(end)?;

            hash_data(buf.as_slice()).to_hex_string()
        };

        Ok(FileArticlePtr(Arc::new(FileArticle{
            uid, file_id, file_name, file_size,
            begin_time: now,
            state: AtomicU8::new(FileStateImpl::Prepair.into_u8()),
        })))
    }

    pub fn with_info(file_id: String, uid: String, file_name: String, file_size: u64, flag: u8) -> Self {
        Self(Arc::new(FileArticle{
            uid, 
            file_id,
            file_name, 
            file_size, 
            begin_time: 0,
            state: AtomicU8::new(flag),
        }))
    }

    pub fn into_resp(&self) -> Box<dyn ToHttpResult> {
        Box::new(FileArticleResp::new(self.file_id().clone()))
    }

    pub fn into_map(&self) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();

        map.insert("file_id".to_string(), self.0.file_id.clone());
        map.insert("uid".to_string(), self.0.uid.clone());
        map.insert("file_name".to_string(), self.0.file_name.clone());
        map.insert("file_size".to_string(), format!("{}", self.0.file_size));

        map
    }

    pub fn file_id(&self) -> &String {
        &self.0.file_id
    }

    pub fn file_name(&self) -> &String {
        &self.0.file_name
    }

    pub fn file_size(&self) -> u64 {
        self.0.file_size
    }

    pub fn state(&self) -> u8 {
        self.0.state.load(Ordering::SeqCst)
        // self.0.state.into_u8()
    }

    pub fn into_nds_file_article(&self) -> NearResult<NdsFileArticle> {
        let state = self.state();
        match FileStateImpl::from(state) {
            FileStateImpl::Finished => Ok(NdsFileArticle{
                file_id: self.file_id().clone(),
                file_path: get_temp_path().join(self.0.file_id.as_str())
            }),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, format!("Cloud not join nds-process, because it has been {}", state)))
        }
    }

}

// impl<'a> Into<BTreeMap<String, String>> for FileArticle<'a> {
//     fn into(self) -> BTreeMap<String, String> {
//         let mut map = BTreeMap::new();

//         map.insert("file_id".to_string(), self.file_id);
//         map.insert("uid".to_string(), self.uid.to_string());
//         map.insert("file_name".to_string(), self.file_name.to_string());
//         map.insert("file_size".to_string(), format!("{}", self.file_size));

//         map
//     }
// }

mod test {

    #[test]
    fn test_filter() {
        use crate::gateway::p::FileArticleManager;
        use super::{FileArticlePtr};

        for i in 0u64..10 {
            let f = 
                FileArticlePtr::new(format!("uid={}", i), format!("filename={}", i), i+1).unwrap();
            let _ = FileArticleManager::get_instance().add_file(f);
        }

        let r = FileArticleManager::get_instance().filter_file(0);
        println!("1: {:?}", r);
        let r = FileArticleManager::get_instance().filter_file(1);
        println!("2: {:?}", r);
        let r = FileArticleManager::get_instance().filter_file(2);
        println!("3: {:?}", r);
        let r = FileArticleManager::get_instance().filter_file(3);
        println!("4: {:?}", r);
        let r = FileArticleManager::get_instance().filter_file(4);
        println!("5: {:?}", r);
    }
}
