
use std::{cell::UnsafeCell, sync::{Arc, RwLock}, path::{Path, PathBuf}, collections::{BTreeMap, btree_map::Entry}, io::Read};

use log::{error, warn};
use near_base::{NearError, ErrorCode, NearResult};
use rlua::{Lua, Function};

use super::configure::ConfigureData;

struct ManagerImpl {
    modules: RwLock<BTreeMap<String, Arc<Lua>>>,
}

pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn new() -> Self {
        Self(Arc::new(ManagerImpl {
            modules: RwLock::new(BTreeMap::new()),
        }))
    }

    fn load_lua(lua_file: PathBuf) -> NearResult<Lua> {
        let lua = Lua::new();

        // load global configure
        lua.context(| ctx | {
            let globals = ctx.globals();
            let userdata = ctx.create_userdata(ConfigureData::get_instace().clone()).unwrap();
            globals.set("configure_data", userdata)
                .map_err(| e | {
                    let error_string = format!("file set userdata with err: {e}");
                    error!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_3RD, error_string)
                })
        })?;

        let data = {
            let mut fs = 
                std::fs::OpenOptions::new()
                    .read(true)
                    .open(lua_file.as_path())
                    .map_err(| e | {
                        let error_string = format!("failed read {} with err: {e}", lua_file.display());
                        error!("{error_string}");
                        NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string)
                    })?;

            let mut data = vec![0u8; fs.metadata().unwrap().len() as usize];
            fs.read_exact(data.as_mut_slice());
            data
        };

        lua.context(| ctx | {
            ctx.load(data.as_slice())
            .exec()
            .map_err(| e | {
                let error_string = format!("failed load {} with err: {e}", lua_file.display());
                error!("{error_string}");
                NearError::new(ErrorCode::NEAR_ERROR_3RD, error_string)
            })
        })?;

        Ok(lua)

    }

    pub fn load(&self, lua_file: PathBuf) -> NearResult<()> {
        let module = 
            lua_file.file_stem()
                .ok_or(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("{} missing file-name.", lua_file.display())))?
                .to_string_lossy()
                .to_string();

        println!("load {module}");

        match self.0.modules.write().unwrap().entry(module.clone()) {
            Entry::Occupied(exist) => 
                Err(NearError::new(ErrorCode::NEAR_ERROR_ALREADY_EXIST, format!("{module} has been exist."))),
            Entry::Vacant(empty) => {
                let lua = Manager::load_lua(lua_file)?;
                empty.insert(Arc::new(lua));
                Ok(())
            }
        }
    }

    pub fn call(&self, module: &str, function: &str) -> NearResult<Vec<u8>> {
        let module = {
            self.0.modules.read().unwrap()
                .get(module)
                .ok_or_else(|| {
                    let error_string = format!("Not found {module} module");
                    warn!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
                })?
                .clone()
        };

        module.context(| ctx | {
            let globals = ctx.globals();
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

}
