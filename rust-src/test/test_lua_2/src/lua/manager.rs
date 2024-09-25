
use std::{sync::Arc, path::PathBuf, collections::{BTreeMap, btree_map::Entry}, io::Read};

use async_std::sync::Mutex;
use log::{error, warn};
use near_base::{NearError, ErrorCode, NearResult};
use rlua::{Lua, Function, FromLuaMulti};

// use crate::tasks::TaskModule;

use super::{configure::ConfigureData, data::Data, built_in::built_inner::BuiltInner, utils};

const BUILT_INNER_DEFAULT_NAME: &'static str = "built-inner.lua";

pub struct InnerLua {
    lua: Mutex<Lua>,
}

impl InnerLua {
    pub async fn test_function(&self, function: &str) -> NearResult<()> {
        self.lua
            .lock().await
            .context(| ctx | {
                let globals = ctx.globals();
                // get function
                globals.get::<_, Function>(function)
                    .map(| _ | (()))
                    .map_err(| e | {
                        let error_string = format!("Not found {function} function");
                        error!("{error_string}");
                        NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
                    })
            })
    }

    pub async fn call(&self, function: &str, params: Data) -> NearResult<Vec<u8>> {
        self.lua
            .lock().await
            .context(| ctx | {
                let globals = ctx.globals();
                // set params
                let tb = 
                    ctx.create_table()
                        .map_err(| e | {
                            let error_string = format!("file create table with err: {e}");
                            error!("{error_string}");
                            NearError::new(ErrorCode::NEAR_ERROR_3RD, error_string)
                        })?;
        
                for (k, v) in params.into_map() {
                    let _ = tb.set(k, v);
                }
                globals.set("thing_data", tb)
                    .map_err(| e | {
                        let error_string = format!("failed set thing_data with err: {e}");
                        error!("{error_string}");
                        NearError::new(ErrorCode::NEAR_ERROR_3RD, error_string)
                    })?;
        
                // get function
                let fun = 
                    globals.get::<_, Function>(function)
                        .map_err(| e | {
                            let error_string = format!("Not found {function} function");
                            error!("{error_string}");
                            NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
                        })?;
        
                fun.call::<_, Vec<u8>>(()).map_err(| e | {
                    let error_string = format!("failed call with err: {e}");
                    error!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_3RD, error_string)
                })
            })
    }

    pub async fn parse(&self, function: &str, data: Vec<u8>, params: Data) -> NearResult<bool> {
        self.lua
            .lock().await
            .context(| ctx | {
                let globals = ctx.globals();

                // userdata
                let userdata = ctx.create_userdata(params).unwrap();
                globals.set("thing_data", userdata)
                    .map_err(| e | {
                        let error_string = format!("file set device_data with err: {e}");
                        error!("{error_string}");
                        NearError::new(ErrorCode::NEAR_ERROR_3RD, error_string)
                    })?;


                // get function
                let fun = 
                globals.get::<_, Function>(function)
                    .map_err(| e | {
                        let error_string = format!("failed get {function} with err: {e}");
                        error!("{error_string}");
                        NearError::new(ErrorCode::NEAR_ERROR_3RD, error_string)
                    })?;

                fun.call::<Vec<u8>, bool>(data).map_err(| e | {
                    let error_string = format!("failed call with err: {e}");
                    error!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_3RD, error_string)
                })
            })
    }

}

impl From<Lua> for InnerLua {
    fn from(value: Lua) -> Self {
        Self {
            lua: Mutex::new(value),
        }
    }
}

struct ManagerImpl {
    modules: BTreeMap<String, InnerLua>,
    built_inner: Option<BuiltInner>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

unsafe impl Send for Manager {}
unsafe impl Sync for Manager {}

impl Manager {
    pub async fn open(workspace: PathBuf) -> NearResult<Self> {
        let file_array = 
        {
            let mut file_array = vec![];

            let dir = 
                std::fs::read_dir(workspace.as_path())
                    .map_err( | e | {
                        let error_string = format!("failed read dir from {} with err: {e}", workspace.display());
                        error!("{error_string}");
                        NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string)
                    })?;

            for file in dir {
                if let Ok(f) = file {
                    let file_path = f.path();

                    if !file_path.is_file() {
                        continue;
                    }

                    if !file_path.extension().unwrap_or_default().eq_ignore_ascii_case("lua") {
                        continue;
                    }

                    if file_path.file_name().unwrap_or_default().eq_ignore_ascii_case(BUILT_INNER_DEFAULT_NAME) {
                        continue;
                    }

                    file_array.push(file_path);
                }
            }

            file_array
        };

        // load built-in lua
        let built_inner = {
            let built_inner = workspace.join(BUILT_INNER_DEFAULT_NAME);
            if built_inner.exists() {
                Some(BuiltInner::open(built_inner).await?)
            } else {
                None
            }
        };

        let mut modules = BTreeMap::new();
        // load lua
        for f in file_array {
            let (module, lua) = Manager::load(f)?;
            modules.insert(module, lua);
        }

        Ok(Self(Arc::new(ManagerImpl{
            built_inner: built_inner,
            modules,
        })))
    }

    fn load(lua_file: PathBuf) -> NearResult<(String, InnerLua)> {
        let module = 
            lua_file.file_stem()
                .ok_or(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("{} missing file-name.", lua_file.display())))?
                .to_string_lossy()
                .to_string();

        Ok((module, utils::load_lua(lua_file)?.into()))
    }

    pub async fn call(&self, module: &str, function: &str, params: Data) -> NearResult<Vec<u8>> {
        let module = {
            self.0.modules.get(module)
                .ok_or_else(|| {
                    let error_string = format!("Not found {module} module");
                    warn!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
                })?
        };

        module.call(function, params).await
    }

}

impl Manager {
    async fn parse_data(&self, module: &str, input: Vec<u8>) -> NearResult<Data> {
        let module = {
            self.0.modules.get(module)
                .ok_or_else(|| {
                    let error_string = format!("Not found {module} module");
                    warn!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
                })?
        };

        let output = Data::default();

        let r = module.parse("alalize_data", input, output.clone()).await?;

        if r { Ok(output) } else { Err(NearError::new(ErrorCode::NEAR_ERROR_3RD, "error")) }
    }

    pub async fn analyze_data(&self, input: Vec<u8>) -> NearResult<Data> {
        let built_inner = 
            self.0.built_inner
                .as_ref()
                .ok_or_else(|| {
                    let error_string = format!("builit-innner lua unload.");
                    error!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
                })?;

        let module = built_inner.analyze(input.clone()).await?;

        self.parse_data(module.as_str(), input).await
    }

}

