
use std::sync::Mutex;

use log::{info, error};
use near_base::{DeviceObjectSubCode, ErrorCode, NearError, NearResult, PrivateKey, };

use crate::{seed_key_bip::SeedKeyBip, path::{ChainObjectType, ChainNetwork, ChainBipPath}};

struct Profile {
    mnemonic: Option<String>,
    password: Option<String>,
    coin: Option<ChainObjectType>,
    network: ChainNetwork,
}

impl Profile {
    fn new() -> Self {
        Self{
            mnemonic: None,
            password: None,
            coin: None,
            network: ChainNetwork::Main,
        }
    }

    fn set_mnemonic(&mut self, mnemonic: Option<&str>) -> NearResult<Option<String>> {
        if let Some(mn) = mnemonic {
            info!("set mnemonic: {:?}", mnemonic);
            self.mnemonic = Some(mn.to_owned());
            Ok(self.mnemonic.clone())
        } else {
            use bip39::*;
            let mn = 
                Mnemonic::generate_in(Language::English, 12).map_err(| e | {
                    let error_string = format!("failed to generate mnemonic with err: {e}");
                    error!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_3RD, error_string)
                })?;
            self.mnemonic = Some(mn.to_string());
            info!("random mnemonic: {}", mn);

            Ok(self.mnemonic.clone())
        }
    }

    fn set_password(&mut self, password: &str) {
        self.password = Some(password.to_owned());
    }

    fn set_test_network(&mut self) {
        self.network = ChainNetwork::Test;
    }

    fn set_beta_network(&mut self) {
        self.network = ChainNetwork::Beta;
    }

    fn set_main_network(&mut self) {
        self.network = ChainNetwork::Main;
    }

    fn set_device_type(&mut self) {
        self.coin = Some(ChainObjectType::Device(near_base::ObjectTypeCode::Device(DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8)));
    }

    fn set_people_type(&mut self) {
        self.coin = Some(ChainObjectType::Device(near_base::ObjectTypeCode::People));
    }
}

impl Profile {
    fn build(self) -> NearResult<PrivateKey> {
       
        let mn = self.mnemonic.as_ref().ok_or_else(||NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing mnemonic string"))?;
        let mnemonic = SeedKeyBip::from_mnemonic(mn, self.password.as_ref().map(| v | v.as_str()))?;
        let coin = self.coin.as_ref().ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing coin"))?;

        let bip_path = 
            match coin {
                ChainObjectType::Device(_) => ChainBipPath::new_device(self.network, 0, Some(0)),
                ChainObjectType::People(_) => ChainBipPath::new_people(self.network, Some(0)),
            };

        let key = 
            match coin {
                ChainObjectType::Device(_) => mnemonic.sub_key(&bip_path, crate::CipPrivateKey::Rsa2048)?,
                ChainObjectType::People(_) => mnemonic.sub_key(&bip_path, crate::CipPrivateKey::Rsa1024)?,
            };

        Ok(key)
    }
}

lazy_static::lazy_static! {
    static ref PROFILE: Mutex<Option<Profile>> = Mutex::new(None);
}

pub fn profile_init() {
    let w = &mut *PROFILE.lock().unwrap();

    if let None = w {
        *w = Some(Profile::new())
    }
}

pub fn set_mnemonic(mnemonic: Option<&str>) -> NearResult<Option<String>> {
    let w = &mut *PROFILE.lock().unwrap();
    let w = w.as_mut().ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_UNINITIALIZED, "uninitialized."))?;

    w.set_mnemonic(mnemonic)
}

pub fn set_password(password: &str) -> NearResult<()> {
    let w = &mut *PROFILE.lock().unwrap();
    let w = w.as_mut().ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_UNINITIALIZED, "uninitialized."))?;
    w.set_password(password);

    Ok(())
}

pub fn set_test_network() -> NearResult<()> {
    let w = &mut *PROFILE.lock().unwrap();
    let w = w.as_mut().ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_UNINITIALIZED, "uninitialized."))?;
    w.set_test_network();

    Ok(())
}

pub fn set_beta_network() -> NearResult<()> {
    let w = &mut *PROFILE.lock().unwrap();
    let w = w.as_mut().ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_UNINITIALIZED, "uninitialized."))?;
    w.set_beta_network();

    Ok(())
}

pub fn set_main_network() -> NearResult<()> {
    let w = &mut *PROFILE.lock().unwrap();
    let w = w.as_mut().ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_UNINITIALIZED, "uninitialized."))?;
    w.set_main_network();

    Ok(())
}

pub fn set_device_type() -> NearResult<()> {
    let w = &mut *PROFILE.lock().unwrap();
    let w = w.as_mut().ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_UNINITIALIZED, "uninitialized."))?;
    w.set_device_type();

    Ok(())
}

pub fn set_people_type() -> NearResult<()> {
    let w = &mut *PROFILE.lock().unwrap();
    let w = w.as_mut().ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_UNINITIALIZED, "uninitialized."))?;
    w.set_people_type();

    Ok(())
}

pub fn build() -> NearResult<PrivateKey> {
    let profile = {
        std::mem::replace(&mut *PROFILE.lock().unwrap(), None)
    }
    .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_UNINITIALIZED, "uninitialized."))?;

    profile.build()
        .map_err(| e | {
            error!("failed build key with err: {e}");
            e
        })
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use near_base::FileEncoder;

    use super::*;

    #[test]
    fn test_cip39() {
        profile_init();
        let phrase = "bar cinnamon grow hungry lens danger treat artist hello seminar document gasp";
        set_mnemonic(Some(phrase)).unwrap();
        set_people_type().unwrap();
        let key = build().unwrap();
        key.encode_to_file(PathBuf::new().join("d:\\1.txt").as_path(), true).unwrap();
    }

}
