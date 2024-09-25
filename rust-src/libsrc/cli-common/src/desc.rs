
pub mod build_pk {
    use near_base::*;

    pub fn set_mnemonic(mnemonic: Option<&str>) -> NearResult<Option<String>> {
        match cip39::profile::set_mnemonic(mnemonic) {
            Ok(v) => Ok(v),
            Err(e) => {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_UNINITIALIZED => {
                        cip39::profile::profile_init();
                        cip39::profile::set_mnemonic(mnemonic)
                    }
                    _ => { Err(e) }
                }
            }
        }
    }

    pub fn set_password(password: String) -> NearResult<()> {
        match cip39::profile::set_password(&password) {
            Ok(v) => Ok(v),
            Err(e) => {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_UNINITIALIZED => {
                        cip39::profile::profile_init();
                        cip39::profile::set_password(&password)
                    }
                    _ => { Err(e) }
                }
            }
        }
    }

    pub fn set_test_network() -> NearResult<()> {
        match cip39::profile::set_test_network() {
            Ok(v) => Ok(v),
            Err(e) => {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_UNINITIALIZED => {
                        cip39::profile::profile_init();
                        cip39::profile::set_test_network()
                    }
                    _ => { Err(e) }
                }
            }
        }
    }

    pub fn set_beta_network() -> NearResult<()> {
        match cip39::profile::set_beta_network() {
            Ok(v) => Ok(v),
            Err(e) => {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_UNINITIALIZED => {
                        cip39::profile::profile_init();
                        cip39::profile::set_beta_network()
                    }
                    _ => { Err(e) }
                }
            }
        }
    }

    pub fn set_main_network() -> NearResult<()> {
        match cip39::profile::set_main_network() {
            Ok(v) => Ok(v),
            Err(e) => {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_UNINITIALIZED => {
                        cip39::profile::profile_init();
                        cip39::profile::set_main_network()
                    }
                    _ => { Err(e) }
                }
            }
        }
    }

    pub fn set_device_type() -> NearResult<()> {
        match cip39::profile::set_device_type() {
            Ok(v) => Ok(v),
            Err(e) => {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_UNINITIALIZED => {
                        cip39::profile::profile_init();
                        cip39::profile::set_device_type()
                    }
                    _ => { Err(e) }
                }
            }
        }
    }

    pub fn set_people_type() -> NearResult<()> {
        match cip39::profile::set_people_type() {
            Ok(v) => Ok(v),
            Err(e) => {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_UNINITIALIZED => {
                        cip39::profile::profile_init();
                        cip39::profile::set_people_type()
                    }
                    _ => { Err(e) }
                }
            }
        }
    }

}

use std::path::PathBuf;
use log::debug;
use near_base::{NearResult, ObjectBuilder, NearError, ErrorCode, FileEncoder};
use near_base::people::{PeopleDescContent, PeopleBodyContent};
use near_util::{DESC_SUFFIX_NAME, KEY_SUFFIX_NAME};

pub fn build(user_name: String, user_data: Vec<u8>, output_dir: String) -> NearResult<String> {
    let output_dir = PathBuf::new().join(output_dir);
    if !output_dir.exists() {
        Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("{} isnot exist.", output_dir.display())))
    } else {
        Ok(())
    }?;

    let _ = cip39::profile::set_people_type()?;

    debug!("build key");
    let key = cip39::profile::build()?;
    let name = if user_name.is_empty() { "BM" } else { user_name.as_str() };

    debug!("build {} desc", name);
    let o =
        ObjectBuilder::new(PeopleDescContent::new(), PeopleBodyContent::default())
            .update_desc(| desc | {
                desc.set_public_key(key.public());
            })
            .update_body(| body | {
                body.mut_body()
                    .set_name(Some(name))
                    .set_userdata(Some(user_data));
            })
            .build()?;

    let id = o.object_id().to_string();

    debug!("encode key-file");
    let _ = key.encode_to_file(output_dir.join(PathBuf::new().with_file_name(id.as_str()).with_extension(KEY_SUFFIX_NAME)).as_path(), true)?;
    debug!("encode desc-file");
    let _ = o.encode_to_file(output_dir.join(PathBuf::new().with_file_name(id.as_str()).with_extension(DESC_SUFFIX_NAME)).as_path(), true)?;

    Ok(id)
}