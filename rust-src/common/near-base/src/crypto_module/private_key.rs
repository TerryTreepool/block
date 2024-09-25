
use std::str::FromStr;

use rsa::{RSAPrivateKey, PublicKeyParts};
use rand::Rng;

use crate::{NearResult, NearError, ErrorCode, hash_data, Serialize, Deserialize, now};

use super::{public_key::PublicKey, signature::{Signature, SignData}};

const KEY_TYPE_RSA: u8 = 1u8;

#[derive(Clone, Copy)]
pub enum PrivateKeyType {
    Rsa,
    Secp256k1,
}

impl PrivateKeyType {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::Rsa => "rsa",
            Self::Secp256k1 => "secp256k1",
        }
    }
}

impl Default for PrivateKeyType {
    fn default() -> Self {
        Self::Rsa
    }
}

impl std::fmt::Display for PrivateKeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for PrivateKeyType {
    type Err = NearError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rsa" => Ok(Self::Rsa),
            "secp256k1" => Ok(Self::Secp256k1),
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("unknown PrivateKey type: {}", s)))
            }
        }
    }
}

#[derive(Clone)]
pub enum PrivateKey {
    Rsa(RSAPrivateKey),
}

impl std::fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[private key *** ***]")
    }
}

impl PrivateKey {
    pub fn key_type(&self) -> PrivateKeyType {
        match *self {
            Self::Rsa(_) => PrivateKeyType::Rsa,
        }
    }

    pub fn generate_rsa1024() -> NearResult<Self> {
        let mut rng = rand::thread_rng();
        Self::generate_rsa(&mut rng, Some(1024))
    }

    pub fn generate_rsa2048() -> NearResult<Self> {
        let mut rng = rand::thread_rng();
        Self::generate_rsa(&mut rng, Some(2048))
    }

    pub fn generate_rsa<R: Rng>(
        rng: &mut R,
        bits: Option<usize>,
    ) -> NearResult<Self> {
        let bits = 
            match bits.unwrap_or(1024) {
                1024 => Ok(1024),
                2048 => Ok(2048),
                _ => { Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "invalid bit")) }
            }?;

        match RSAPrivateKey::new(rng, bits) {
            Ok(v) => { Ok(Self::Rsa(v)) }
            Err(e) => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_GENRSA, format!("failed generate private key with error string {}", e.to_string()))) }
        }
    }

    pub fn public(&self) -> PublicKey {
        match self {
            Self::Rsa(private_key) => PublicKey::Rsa(private_key.to_public_key()),
        }
    }
}

impl PrivateKey {
    pub fn decrypt(&self, input: &[u8], output: &mut [u8]) -> NearResult<usize> {
        match self {
            Self::Rsa(private_key) => {
                let buf =
                    private_key.decrypt(rsa::PaddingScheme::PKCS1v15Encrypt, input)
                               .map_err(|e| {
                                   NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_DECRYPT,
                                                  format!("failed decrypt buf with error {}", e.to_string()))
                               })?;
                if output.len() < buf.len() {
                    Err(NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_DECRYPT,
                                       format!("not enough buffer, rsa decrypt error, except={}, get={}",
                                                        output.len(), buf.len())))
                } else {
                    let size = buf.len();
                    output[..size].copy_from_slice(buf.as_slice());
                    Ok(size)
                }
            }
        }
    }

    pub fn sign(&self, data: &[u8]) -> NearResult<Signature> {
        // 签名必须也包含签名的时刻，这个时刻是敏感的不可修改
        let sign_time = now();

        let data_new = [
            data,
            &sign_time.to_be_bytes(),
        ].concat();

        match self {
            Self::Rsa(private_key) => {
                let hash = hash_data(data_new.as_slice());
                let sign =
                    private_key.sign(rsa::PaddingScheme::new_pkcs1v15_sign(Some(rsa::Hash::SHA2_256)),
                                     hash.as_slice())
                               .map_err(|e| {
                                    NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_SIGN,
                                                   format!("failed signature with error {}", e.to_string()))
                               })?;
                assert_eq!(sign.len(), private_key.size());
                let sign_data = SignData::try_from(sign.as_slice())?;

                Ok(Signature::new(sign_time, sign_data))
            }
        }
    }
}

impl Serialize for PrivateKey {
    fn raw_capacity(&self) -> usize {
        match self {
            Self::Rsa(rsa) => {
                if let Ok(spki_der) = rsa_export::pkcs1::private_key(rsa) {
                    return KEY_TYPE_RSA.raw_capacity() +
                    spki_der.raw_capacity();
                } else {
                    return 0;
                }
            }
        }
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        match self {
            Self::Rsa(rsa) => {
                let buf = KEY_TYPE_RSA.serialize(buf)?;
                let buf =
                    rsa_export::pkcs1::private_key(rsa)
                        .map_err(| err | {
                            NearError::new(ErrorCode::NEAR_ERROR_3RD, format!("failed get private-key with err={}", err))
                        })?
                        .serialize(buf)?;

                Ok(buf)
            }
        }
    }
}

impl Deserialize for PrivateKey {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (ty, buf) = u8::deserialize(buf)?;

        match ty {
            KEY_TYPE_RSA => {
                let (key_data, buf) = Vec::<u8>::deserialize(buf)?;
                let private_key = rsa::RSAPrivateKey::from_pkcs1(key_data.as_slice())
                                                    .map_err(| e | {
                                                        NearError::new(ErrorCode::NEAR_ERROR_3RD, format!("Failed import rsa key with err={e}"))
                                                    })?;

                Ok((Self::Rsa(private_key), buf))
            }
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("Undefined identification code, except={ty}")))
            }
        }
    }
}
