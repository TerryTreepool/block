
use std::{sync::Arc, path::PathBuf, io::Read};

use log::error;
use near_base::{NearResult, NearError, ErrorCode};
use rlua::Function;

use crate::lua::{utils, manager::InnerLua, data::Data};

pub struct BuiltInner {
    lua: InnerLua,
}

impl BuiltInner {
    pub async fn open(built_in_file: PathBuf) -> NearResult<Self> {

        let lua: InnerLua = utils::load_lua(built_in_file)?.into();

        lua.test_function("alalize_data").await?;

        Ok(Self{lua})
    }

}

impl BuiltInner {
    pub async fn analyze(&self, data: Vec<u8>) -> NearResult<String> {
        let params = Data::default();

        if self.lua.parse("alalize_data", data, params.clone()).await? {
            params.get("module")
                .ok_or_else(|| {
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not found **module key field")
                })
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "execute alalize_data failure."))
        }
    }
}
