
use std::{path::PathBuf, io::Read};

use log::error;
use near_base::{NearResult, NearError, ErrorCode};

use rlua::Lua;

use super::configure::ConfigureData;

pub fn load_lua(lua_file: PathBuf) -> NearResult<Lua> {
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
        fs.read_exact(data.as_mut_slice())
            .map_err(| e | {
                let error_string = format!("failed read_exact {} with err: {e}", lua_file.display());
                error!("{error_string}");
                NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string)
            })?;
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
