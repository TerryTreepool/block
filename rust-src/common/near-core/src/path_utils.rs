
use std::{path::PathBuf,
          sync::{Mutex, MutexGuard},
          collections::BTreeMap, 
    };

use log::error;
use near_base::{NearError, NearResult};

#[cfg(target_os = "windows")]
const EXTENTION_SUFFIX: &str = "dll";
#[cfg(any(target_os = "linux", target_os = "android", target_os = "ios"))]
const EXTENTION_SUFFIX: &str = "so";
#[cfg(target_os = "macos")]
const EXTENTION_SUFFIX: &str = "dylib";

pub const NEAR_ROOT_NAME: &str = "block-magic";

fn default_near_root_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(&format!("C:\\{}", NEAR_ROOT_NAME))
    }

    #[cfg(target_os = "android")]
    {
        PathBuf::from(&format!("/storage/emulated/0/{}", NEAR_ROOT_NAME))
    }

    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "ios"))]
    {
        match dirs::home_dir() {
            Some(dir) => {
                let root = dir.join(&format!("{NEAR_ROOT_NAME}"));
                if root.is_dir() {
                    root.canonicalize().unwrap()
                } else {
                    root
                }
            }
            None => {
                error!("get user dir failed!");
                PathBuf::from(&format!("/{}", NEAR_ROOT_NAME))
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        PathBuf::new()
    }
}

pub struct NearCatalog {
    root: PathBuf,
    temp_path: PathBuf,
    data_path: PathBuf,
    cache_path: PathBuf,
    log_path: PathBuf,
    bin_path: PathBuf,
    app_path: PathBuf,
    app_path_list: BTreeMap<String, PathBuf>,
}

const NEAR_HOME: &str = "NEAR_HOME";

const TEMP_LABEL: &str = "temp";
const DATA_LABEL: &str = "data";
const CACHE_LABEL: &str = "cache";
const LOG_LABEL: &str = "log";
const APP_LABEL: &str = "app";
const BIN_LABEL: &str = "bin";

impl std::default::Default for NearCatalog {
    fn default() -> Self {
        Self {
            root: PathBuf::default(),
            temp_path: PathBuf::default(),
            data_path: PathBuf::default(),
            cache_path: PathBuf::default(),
            log_path: PathBuf::default(),
            bin_path: PathBuf::default(),
            app_path: PathBuf::default(),
            app_path_list: BTreeMap::new(),
        }
    }
}

impl NearCatalog {
    fn new() -> NearResult<Self> {
        NearCatalog::with_root(&{
            std::env::var(NEAR_HOME)
                .map(| value | {
                    println!("value={value}");
                    PathBuf::new().join(value)
                })
                .unwrap_or(default_near_root_path())
        })
    }

    fn with_root(root: &PathBuf) -> NearResult<Self> {
        let create_dir = | path: PathBuf | -> NearResult<PathBuf> {
            match std::fs::create_dir_all(&path) {
                Ok(_) => { Ok(path) }
                Err(err) => {
                    error!("failed create_dir {} with err {}", path.display(), err);
                    Err(NearError::from(err))
                }
            }
        };

        let root_path = create_dir(root.clone())?;
        let temp_path = create_dir(PathBuf::from(root).join(TEMP_LABEL))?;
        let data_path = create_dir(PathBuf::from(root).join(DATA_LABEL))?;
        let cache_path = create_dir(PathBuf::from(root).join(CACHE_LABEL))?;
        let log_path = create_dir(PathBuf::from(root).join(LOG_LABEL))?;
        let bin_path = create_dir(PathBuf::from(root).join(BIN_LABEL))?;
        let app_path = create_dir(PathBuf::from(root).join(APP_LABEL))?;

        Ok(Self{
            root: root_path,
            temp_path,
            data_path,
            cache_path,
            log_path,
            bin_path,
            app_path,
            app_path_list: BTreeMap::new(),
        })
    }
}

lazy_static::lazy_static! {
    static ref NEAR_CATALOG: Mutex<Option<NearCatalog>> = Mutex::new(None);
}

fn get_locker() -> MutexGuard<'static, Option<NearCatalog>> {
    NEAR_CATALOG.lock().unwrap()
}

pub fn alter_near_path() -> NearResult<()> {
    let catalog = NearCatalog::new()?;

    let mut locker = get_locker();
    let locker_log = &mut *locker;

    std::mem::swap(locker_log, &mut Some(catalog));

    Ok(())
}

pub fn alter_root_path(root: PathBuf) -> NearResult<()> {
    let catalog = NearCatalog::with_root(&root)?;

    let mut locker = get_locker();
    let locker_log = &mut *locker;

    std::mem::swap(locker_log, &mut Some(catalog));

    Ok(())
}

pub fn get_root_path() -> PathBuf {
    let mut locker = get_locker();
    let locker_log = &mut *locker;

    if let None = locker_log {
        let _ = std::mem::replace(locker_log, Some(NearCatalog::new().unwrap()));
    }

    locker_log.as_ref().unwrap().root.clone()
}

pub fn get_temp_path() -> PathBuf {
    let mut locker = get_locker();
    let locker_log = &mut *locker;

    if let None = locker_log {
        let _ = std::mem::replace(locker_log, Some(NearCatalog::new().unwrap()));
    }

    locker_log.as_ref().unwrap().temp_path.clone()
    // NEAR_CATALOG.lock().unwrap().temp_path.clone()
}

pub fn get_data_path() -> PathBuf {
    let mut locker = get_locker();
    let locker_log = &mut *locker;

    if let None = locker_log {
        let _ = std::mem::replace(locker_log, Some(NearCatalog::new().unwrap()));
    }

    locker_log.as_ref().unwrap().data_path.clone()
    // NEAR_CATALOG.lock().unwrap().data_path.clone()
}

pub fn get_cache_path() -> PathBuf {
    let mut locker = get_locker();
    let locker_log = &mut *locker;

    if let None = locker_log {
        let _ = std::mem::replace(locker_log, Some(NearCatalog::new().unwrap()));
    }

    locker_log.as_ref().unwrap().cache_path.clone()
    // NEAR_CATALOG.lock().unwrap().cache_path.clone()
}

pub fn get_log_path() -> PathBuf {
    let mut locker = get_locker();
    let locker_log = &mut *locker;

    if let None = locker_log {
        let _ = std::mem::replace(locker_log, Some(NearCatalog::new().unwrap()));
    }

    locker_log.as_ref().unwrap().log_path.clone()
    // NEAR_CATALOG.lock().unwrap().log_path.clone()
}

pub fn get_bin_path() -> PathBuf {
    let mut locker = get_locker();
    let locker_log = &mut *locker;

    if let None = locker_log {
        let _ = std::mem::replace(locker_log, Some(NearCatalog::new().unwrap()));
    }

    locker_log.as_ref().unwrap().bin_path.clone()
    // NEAR_CATALOG.lock().unwrap().bin_path.clone()
}

pub fn get_extention_path(extention_name: &str) -> PathBuf {
    get_bin_path().join(format!("{}.{EXTENTION_SUFFIX}", extention_name))
}

pub fn get_app_path() -> PathBuf {
    let mut locker = get_locker();
    let locker_log = &mut *locker;

    if let None = locker_log {
        let _ = std::mem::replace(locker_log, Some(NearCatalog::new().unwrap()));
    }

    locker_log.as_ref().unwrap().app_path.clone()
    // NEAR_CATALOG.lock().unwrap().app_path.clone()
}

pub fn get_service_path(service_name: impl Into<String>) -> PathBuf {
    let mut locker = get_locker();
    let locker_log = &mut *locker;

    if let None = locker_log {
        let _ = std::mem::replace(locker_log, Some(NearCatalog::new().unwrap()));
    }

    let service_name = service_name.into();

    let (path, newly) = {
        let r = locker_log.as_ref().unwrap();
        if let Some(path) = r.app_path_list.get(&service_name) {
            (path.clone(), false)
        } else {
            let path = PathBuf::from(&r.app_path).join(service_name.as_str());

            match std::fs::create_dir_all(&path) {
                Ok(_) => {
                    (path, true)
                }
                Err(_) => (PathBuf::default(), false)
            }
        }
    };

    if newly {
        locker_log.as_mut().unwrap().app_path_list.insert(service_name, path.clone());
        path
    } else {
        path
    }
}

mod test {
    #[test]
    fn test_path() {

        let root = super::get_root_path();
        let temp = super::get_temp_path();
        let data = super::get_data_path();
        let log = super::get_log_path();
        let app = super::get_log_path();
        let ttt = super::get_service_path("ttt");

        println!("orig: {}", root.display());
        println!("orig: {}", temp.display());
        println!("orig: {}", data.display());
        println!("orig: {}", log.display());
        println!("orig: {}", app.display());
        println!("orig: {}", ttt.display());

        let _ = super::alter_root_path(std::path::PathBuf::from("e:/root"));

        let root = super::get_root_path();
        let temp = super::get_temp_path();
        let data = super::get_data_path();
        let log = super::get_log_path();
        let app = super::get_log_path();
        let ttt = super::get_service_path("ttt");

        println!("alter: {}", root.display());
        println!("alter: {}", temp.display());
        println!("alter: {}", data.display());
        println!("alter: {}", log.display());
        println!("alter: {}", app.display());
        println!("alter: {}", ttt.display());
    }
}

