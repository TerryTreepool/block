
use std::path::PathBuf;

use near_base::{NearResult, NearError, ErrorCode};

use crate::{lua::{utils, manager::InnerLua, data::Data}, tasks::TaskModule};

pub struct BuiltInner {
    lua: InnerLua,
}

impl BuiltInner {
    pub async fn open(built_in_file: PathBuf) -> NearResult<Self> {

        let lua: InnerLua = utils::load_lua(built_in_file)?.into();

        lua.test_function(TaskModule::AnalizeData.to_str()).await?;

        Ok(Self{lua})
    }

}

impl BuiltInner {
    pub async fn analyze(&self, data: Vec<u8>) -> NearResult<String> {
        let params = Data::default();

        if self.lua.parse(TaskModule::AnalizeData.to_str(), data, Default::default(), params.clone()).await? {
            params.get("module")
                .ok_or_else(|| {
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not found **module key field")
                })
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "execute TaskModule::AnalizeData failure."))
        }
    }
}
