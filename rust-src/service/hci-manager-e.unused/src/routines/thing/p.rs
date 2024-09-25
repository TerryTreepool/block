
use std::path::PathBuf;

use near_base::{NearResult, thing::ThingObject, FileDecoder};

use crate::{process::Process, public::CheckTrait};

pub(crate) struct CheckAndGetThingObject;

impl CheckAndGetThingObject {
    pub async fn call(process: &Process, path: &PathBuf, thing_id: &str) -> NearResult<ThingObject> {
        crate::public::thing::get_thing(process.db_helper(), thing_id)
            .await?
            .check_status()?;
    
        ThingObject::decode_from_file(
            path.join(PathBuf::new().with_file_name(thing_id)
                                          .with_extension("desc"))
                .as_path())
    }
}
