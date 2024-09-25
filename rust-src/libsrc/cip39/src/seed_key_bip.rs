
use std::fmt;

use bip39::{Language, Mnemonic};
use log::{error, debug};
use memzero::Memzero;
use near_base::{NearResult, NearError, ErrorCode, PrivateKey};

use crate::{seed::Seed, cip39::ExtendedPrivateKey, path::ChainBipPath, CipPrivateKey};

pub struct SeedKeyBip {
    seed: Memzero<[u8; 64]>,
}

impl fmt::Debug for SeedKeyBip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Protected SeedKeyBip]")
    }
}

impl SeedKeyBip {
    pub fn fix_mnemonic(mnemonic: &str) -> NearResult<String> {
        let words: Vec<&str> = mnemonic.split(" ").into_iter().map(| v | v.trim()).collect();
        if words.len() != 12 {
            let msg = format!("invalid mnemonic words: len={}", words.len());
            error!("{}", msg);
            Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, msg))
        } else {
            Ok(())
        }?;

        let mnemonic = words.join(" ");

        Ok(mnemonic)
    }

    pub fn from_mnemonic(mnemonic: &str, password: Option<&str>) -> NearResult<Self> {
        let mnemonic = Self::fix_mnemonic(mnemonic)?;

        let mnemonic = 
            Mnemonic::parse_in_normalized(Language::English, mnemonic.as_str())
                .map_err(|e| {
                    let msg = format!("invalid mnemonic: err={}", e);
                    error!("{}", msg);

                    NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, msg)
                })?;

        let seed_key = Seed::new(&mnemonic, password.unwrap_or(""))?;

        // 64bytes
        let buf: [u8; 64] = seed_key
            .as_bytes()
            .try_into()
            .expect("invalid seed key length!");

        let seed: Memzero<[u8; 64]> = Memzero::<[u8; 64]>::from(buf);
        Ok(Self { seed })
    }

    pub fn from_private_key(private_key: &str, password: &str) -> NearResult<Self> {
        // device的密钥使用peopleId作为password
        let seed_key = Seed::new_from_private_key(private_key, password)?;

        // 64bytes
        let buf: [u8; 64] = seed_key
            .as_bytes()
            .try_into()
            .expect("invalid seed key length!");

        let seed: Memzero<[u8; 64]> = Memzero::<[u8; 64]>::from(buf);
        Ok(Self { seed })
    }

    pub fn from_string(s: &str, password: Option<&str>) -> NearResult<Self> {
        let password = password.unwrap_or("");

        let seed_key = Seed::new_from_string(s, password)?;

        // 64bytes
        let buf: [u8; 64] = 
            seed_key
                .as_bytes()
                .try_into()
                .map_err(| _e | {
                    NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, "invalid seed key length!")
                })?;

        let seed: Memzero<[u8; 64]> = Memzero::<[u8; 64]>::from(buf);
        Ok(Self { seed })
    }

    pub fn sub_key(
        &self,
        path: &ChainBipPath,
        key: CipPrivateKey,
    ) -> NearResult<PrivateKey> {
        let path = path.to_string();
        debug!("will derive private key by path={}, key={}", path, key);

        let epk = ExtendedPrivateKey::derive(self.seed.as_ref(), path.as_str(), key)?;
        Ok(epk.into())
    }

    // // 直接从path来生成子密钥, 对path合法性不做检测
    // pub fn sub_key_direct_by_path(
    //     &self,
    //     path: &str,
    // ) -> BuckyResult<PrivateKey> {
    //     self.sub_key_direct_by_path_ex(path, PrivateKeyType::Rsa, None)
    // }

    // pub fn sub_key_direct_by_path_ex(
    //     &self,
    //     path: &str,
    //     pt: PrivateKeyType,
    //     bits: Option<usize>,
    // ) -> BuckyResult<PrivateKey> {
    //     debug!(
    //         "will derive private key direct by path={}, type={}, bits={:?}",
    //         path, pt, bits
    //     );

    //     let epk = ExtendedPrivateKey::derive(self.seed.as_ref(), path, pt, bits)?;
    //     Ok(epk.into())
    // }
}
