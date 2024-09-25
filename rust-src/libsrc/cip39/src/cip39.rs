
use std::fmt;
use std::ops::Deref;

use hmac::{Hmac, Mac,};
use log::error;
use memzero::Memzero;
use near_base::{NearResult, PrivateKey, NearError, ErrorCode, RawConvertTo};
use rand_chacha::rand_core::SeedableRng;
use sha2::Sha512;

use crate::{cip39_p::{ChildNumber, IntoDerivationPath}, CipPrivateKey};

struct PrivateKeySeedGen;

impl PrivateKeySeedGen {
    pub fn gen(seed: &[u8], key: CipPrivateKey) -> NearResult<PrivateKey> {
        assert!(seed.len() == 32);

        // let mut rng: PBKDF2Rng = crate::pbkdf2_rand::PBKDF2Rng::from_seed(seed.try_into().expect("invalid seed bytes length"));
        let mut rng = 
            rand_chacha::ChaCha20Rng::from_seed(
                seed.try_into().map_err(| _e | {
                    NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "invalid seed bytes length")
                })?
            );

        match key {
            CipPrivateKey::Rsa1024 => PrivateKey::generate_rsa(&mut rng, Some(1024)),
            CipPrivateKey::Rsa2048 => PrivateKey::generate_rsa(&mut rng, Some(2048)),
            // PrivateKeyType::Secp256k1 => Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, "undefined secp256k1"))
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Protected(Memzero<[u8; 32]>);

impl<Data: AsRef<[u8]>> From<Data> for Protected {
    fn from(data: Data) -> Protected {
        let mut buf = [0u8; 32];

        buf.copy_from_slice(data.as_ref());

        Protected(Memzero::from(buf))
    }
}

impl Deref for Protected {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl fmt::Debug for Protected {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Protected]")
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ExtendedPrivateKey {
    private_key: PrivateKey,
    chain_code: Protected,
}

impl ExtendedPrivateKey {
    /// Attempts to derive an extended private key from a path.
    pub fn derive<Path>(seed: &[u8], path: Path, key: CipPrivateKey) -> NearResult<ExtendedPrivateKey>
    where
        Path: IntoDerivationPath,
    {
        let mut hmac: Hmac<Sha512> = Hmac::new_from_slice(b"BFC seed").unwrap();
        hmac.update(seed);

        let result = hmac.finalize().into_bytes();
        let (private_key, chain_code) = result.split_at(32);

        let mut sk = ExtendedPrivateKey {
            private_key: PrivateKeySeedGen::gen(&private_key, key)?,
            chain_code: Protected::from(chain_code),
        };

        for child in path.into()?.as_ref() {
            sk = sk.child(*child, key)?;
        }

        Ok(sk)
    }

    pub fn secret(&self) -> &PrivateKey {
        &self.private_key
    }

    pub fn child(&self, child: ChildNumber, key: CipPrivateKey) -> NearResult<ExtendedPrivateKey> {
        let mut hmac: Hmac<Sha512> = Hmac::new_from_slice(&self.chain_code).map_err(|e| {
            let error_string = format!("invalid chain code, err={}", e);
            error!("{}", error_string);
            NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, error_string)
        })?;

        if child.is_normal() {
            let bytes = self.private_key.public().to_vec()?;
            hmac.update(&bytes[..]);
        } else {
            let bytes = self.private_key.to_vec()?;

            hmac.update(&[0]);
            hmac.update(&bytes[..]);
        }

        hmac.update(&child.to_bytes());

        let result = hmac.finalize().into_bytes();
        let (private_key, chain_code) = result.split_at(32);

        Ok(ExtendedPrivateKey {
            private_key: PrivateKeySeedGen::gen(&private_key, key)?,
            chain_code: Protected::from(&chain_code),
        })
    }
}

impl Into<PrivateKey> for ExtendedPrivateKey {
    fn into(self) -> PrivateKey {
        self.private_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use bip39::rand;
    use rand_chacha::rand_core::RngCore;

    #[test]
    fn test_rng() {
        let mut rng = rand::thread_rng();
        let mut bytes = vec![0u8; 32];
    
        rng.fill_bytes(&mut bytes);
        println!("seed: {}", hex::encode(&bytes));

        let pk = PrivateKeySeedGen::gen(&bytes, CipPrivateKey::Rsa1024).unwrap();
        let buf = pk.to_vec().unwrap();
        
        println!("sk: {}", hex::encode(&buf));

        let pk = PrivateKeySeedGen::gen(&bytes, CipPrivateKey::Rsa1024).unwrap();
        let buf = pk.to_vec().unwrap();
        
        println!("sk: {}", hex::encode(&buf));

    }
    /*
    //#[test]
    fn bip39_to_address() {
        let phrase = "panda eyebrow bullet gorilla call smoke muffin taste mesh discover soft ostrich alcohol speed nation flash devote level hobby quick inner drive ghost inside";

        let expected_secret_key = b"\xff\x1e\x68\xeb\x7b\xf2\xf4\x86\x51\xc4\x7e\xf0\x17\x7e\xb8\x15\x85\x73\x22\x25\x7c\x58\x94\xbb\x4c\xfd\x11\x76\xc9\x98\x93\x14";
        let expected_address: &[u8] =
            b"\x63\xF9\xA9\x2D\x8D\x61\xb4\x8a\x9f\xFF\x8d\x58\x08\x04\x25\xA3\x01\x2d\x05\xC8";

        let mnemonic = Mnemonic::from_phrase(phrase, Language::English).unwrap();
        let seed = Seed::new(&mnemonic, "");

        let account = ExtendedPrivateKey::derive(seed.as_bytes(), "m/44'/60'/0'/0/0").unwrap();

        assert_eq!(
            expected_secret_key,
            &account.secret(),
            "Secret key is invalid"
        );

        let private_key = SecretKey::from_raw(&account.secret()).unwrap();
        let public_key = private_key.public();

        assert_eq!(expected_address, public_key.address(), "Address is invalid");

        // Test child method
        let account = ExtendedPrivateKey::use rand::RngCore;(seed.as_bytes(), "m/44'/60'/0'/0")
            .unwrap()
            .child(ChildNumber::from_str("0").unwrap())
            .unwrap();

        assert_eq!(
            expected_secret_key,
            &account.secret(),
            "Secret key is invalid"
        );

        let private_key = SecretKey::from_raw(&account.secret()).unwrap();
        let public_key = private_key.public();

        assert_eq!(expected_address, public_key.address(), "Address is invalid");
    }
    */
}
