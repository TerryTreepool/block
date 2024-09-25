

use std::io::Write;

use generic_array::{typenum::U8, GenericArray};
use rand::random;
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit, Unsigned};
use sha2::Digest;

use crate::{Deserialize, ErrorCode, NearError, NearResult, RawFixedBytes, Serialize };

const KEY_LENGTH_DEFAULT_MAX: usize = 32;
const IV_LENGTH_DEFAULT_MAX: usize = 16;

#[derive(Clone, Copy)]
pub struct AesKey {
    key: [u8; KEY_LENGTH_DEFAULT_MAX],
    iv: [u8; IV_LENGTH_DEFAULT_MAX],
}

impl std::default::Default for AesKey {
    fn default() -> Self {
        Self {
            key: [0u8; KEY_LENGTH_DEFAULT_MAX],
            iv: [0u8; IV_LENGTH_DEFAULT_MAX],
        }
    }
}

impl AesKey {
    pub fn generate() -> Self {
        let random_data = | data: &mut [u8], size: usize | {
            let uint = std::mem::size_of::<u32>();
            let count = size / uint;

            if count == 0 {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_NOT_ENOUGH, "not enough space."));
            }

            for i in 0..count {
                data[i*uint..(i+1)*uint].copy_from_slice(&random::<u32>().to_be_bytes());
            }

            Ok(())
        };

        let mut this = Self::default();

        let _ = random_data(&mut this.key, KEY_LENGTH_DEFAULT_MAX);
        let _ = random_data(&mut this.iv, IV_LENGTH_DEFAULT_MAX);

        this
    }

    pub fn mix_hash(&self, salt: Option<u64>) -> KeyMixHash {
        let mut sha = sha2::Sha256::new();
        let _ = sha.write(&self.key);
        let _ = sha.write(&self.iv);
        if let Some(salt) = salt {
            let _ = sha.write(&salt.to_le_bytes());
        }

        let hash = sha.finalize();
        let mut mix_hash =
            GenericArray::from_slice(&hash.as_slice()[..KeyMixHash::raw_bytes()]).clone();
        mix_hash[0] = mix_hash[0] & 0x7f;
        KeyMixHash(mix_hash)
    }


}

impl AesKey {
    pub fn encrypt(&self, data: &[u8], output: &mut [u8]) -> NearResult<usize> {
        // rust_cry

        // #[cfg(not(use_rust_crypto))]
        // {
        //     unimplemented!()
        // }

        // #[cfg(use_rust_crypto)]
        {
            let cipher = cbc::Encryptor::<aes::Aes256>::new((&self.key).into(), (&self.iv).into());

            cipher.encrypt_padded_b2b_mut::<Pkcs7>(data, output)
                .map(| remain_data | remain_data.len() )
                .map_err(| err | {
                    NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_AEK_ENCRYPT, format!("failed cbc_encryptor with error {:?}", err))
                })
        }
    }

    pub fn decrypt<'a>(&self, data: &[u8], output: &'a mut [u8]) -> NearResult<usize> {
        // #[cfg(not(use_rust_crypto))]
        // {
        //     unimplemented!()
        // }

        // #[cfg(use_rust_crypto)]
        {
            let cipher = cbc::Decryptor::<aes::Aes256>::new((&self.key).into(), (&self.iv).into());

            cipher.decrypt_padded_b2b_mut::<Pkcs7>(data, output)
                .map(| remain_data | remain_data.len() )
                .map_err(| err | {
                    NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_AEK_ENCRYPT, format!("failed cbc_decrypt with error {:?}", err))
                })
        }
    }
}

impl Serialize for AesKey {
    fn raw_capacity(&self) -> usize {
        self.key.raw_capacity() + 
        self.iv.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.key.serialize(buf)?;
        let buf = self.iv.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for AesKey {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (key, buf) = Vec::<u8>::deserialize(buf)?;
        let (iv, buf) = Vec::<u8>::deserialize(buf)?;

        Ok((Self{
            key: {
                let mut v = [0u8; KEY_LENGTH_DEFAULT_MAX];
                v.copy_from_slice(key.as_slice());
                v
            }, 
            iv: {
                let mut v = [0u8; IV_LENGTH_DEFAULT_MAX];
                v.copy_from_slice(iv.as_slice());
                v
            }
        }, buf))
    }
}

// aes key 的mixhash
#[derive(Eq, PartialEq, Hash, Clone, Ord, PartialOrd, Debug, Default)]
pub struct KeyMixHash(GenericArray<u8, U8>);

impl AsRef<GenericArray<u8, U8>> for KeyMixHash {
    fn as_ref(&self) -> &GenericArray<u8, U8> {
        &self.0
    }
}

impl AsMut<GenericArray<u8, U8>> for KeyMixHash {
    fn as_mut(&mut self) -> &mut GenericArray<u8, U8> {
        &mut self.0
    }
}

impl std::fmt::Display for KeyMixHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0.as_slice()))
    }
}

impl RawFixedBytes for KeyMixHash {
    fn raw_bytes() -> usize {
        U8::to_usize()
    }
}

impl Serialize for KeyMixHash {
    fn raw_capacity(&self) -> usize {
        self.0.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        self.0.serialize(buf)
    }
}

impl Deserialize for KeyMixHash {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (key, buf) = GenericArray::<u8, U8>::deserialize(buf)?;

        Ok((Self(key), buf))

    }
}

#[cfg(test)]
mod test {
    use crate::{aes_key::KeyMixHash, RawFixedBytes};

    use super::AesKey;

    #[test]
    fn test_aes() {
        let key = AesKey::generate();

        let hash = key.mix_hash(None);
        println!("hash: {hash}, hash-size: {}", KeyMixHash::raw_bytes());

        let mut output = [0u8; 1024];

        let r1 = key.encrypt("123456789".as_bytes(), &mut output).unwrap();

        println!("encrypt: {}", hex::encode(&output[..r1]));

        let mut text = [0u8; 1024];
        let r2 = key.decrypt(&output[..r1], &mut text).unwrap();
        println!("decrypt: {}", String::from_utf8_lossy(&text[..r2]));
    }
}
    