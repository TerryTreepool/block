
use rsa::{RSAPublicKey, PublicKey as RSAPublicKeyTrait, PublicKeyParts};

use crate::errors::{NearResult, NearError, ErrorCode};
use crate::{Serialize, Deserialize, hash_data, };

use super::signature::Signature;

const RSA1024_PUBLIC_CODE: u8 = 1;
const RSA1024_PUBLIC_LENGHT_DEFAULT_MAX: usize = 162;
const RSA2048_PUBLIC_CODE: u8 = 2;
const RSA2048_PUBLIC_LENGHT_DEFAULT_MAX: usize = 294;
const RSA3072_PUBLIC_CODE: u8 = 3;
const RSA3072_PUBLIC_LENGHT_DEFAULT_MAX: usize = 422;

#[derive(Clone)]
pub enum PublicKey {
    Rsa(RSAPublicKey),
}

impl PublicKey {
    pub fn encrypt(&self, data: &[u8], output: &mut [u8]) -> NearResult<usize> {
        match self {
            Self::Rsa(public_key) => {
                let mut rng = rand::thread_rng();
                let encrypted_buf = 
                    public_key.encrypt(&mut rng, rsa::PaddingScheme::PKCS1v15Encrypt, data)
                              .map_err(|e| {
                                    NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_ENCRYPT, 
                                                   format!("failed encrypt with error {}", e.to_string()))})?;
                if output.len() < encrypted_buf.len() {
                    Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, 
                                       format!("not enough buffer, rsa encrypt error, except={}, get={}",
                                                        output.len(), encrypted_buf.len())))
                } else {
                    let size = encrypted_buf.len();
                    output[..size].copy_from_slice(encrypted_buf.as_slice());
                    Ok(size)
                }
            }
        }
    }

    pub fn verify(&self, data: &[u8], sign: &Signature) -> NearResult<()> {
        let sign_time = sign.sign_time();

        let data_new = [
            data,
            &sign_time.to_be_bytes(),
        ].concat();

        match self {
            Self::Rsa(key) => {
                let hash = hash_data(data_new.as_slice());
                key.verify(rsa::PaddingScheme::new_pkcs1v15_sign(Some(rsa::Hash::SHA2_256)),
                           hash.as_slice(),
                           sign.sign_data().as_slice())
                   .map_err(| e | {
                    NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_VERIFY, 
                                    format!("failed verify with error: {}", e.to_string()))
                   })
            }
        }
    }

}

impl Serialize for PublicKey {
    fn raw_capacity(&self) -> usize {
        match self {
            Self::Rsa(pk) => {
                let (code, len) = {
                    match pk.size() {
                    // 1024 bits, 128 bytes
                    128 => (RSA1024_PUBLIC_CODE, RSA1024_PUBLIC_LENGHT_DEFAULT_MAX),
                    256 => (RSA2048_PUBLIC_CODE, RSA2048_PUBLIC_LENGHT_DEFAULT_MAX),
                    384 => (RSA3072_PUBLIC_CODE, RSA3072_PUBLIC_LENGHT_DEFAULT_MAX),
                    _ => { unreachable!() }
                    }
                };
                code.raw_capacity() + len
            }
        }
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        match self {
            Self::Rsa(ref pk) => {
                let (code, len) = {
                    match pk.size() {
                        128 => Ok((RSA1024_PUBLIC_CODE, RSA1024_PUBLIC_LENGHT_DEFAULT_MAX)),
                        256 => Ok((RSA2048_PUBLIC_CODE, RSA2048_PUBLIC_LENGHT_DEFAULT_MAX)),
                        384 => Ok((RSA3072_PUBLIC_CODE, RSA3072_PUBLIC_LENGHT_DEFAULT_MAX)),
                        _ => Err(NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_INVALID_PUBKEY, format!("invalid rsa public key length, except={}", pk.size()))),
                    }
                }?;

                let spki_der = match rsa_export::pkcs1::public_key(pk) {
                    Ok(public_key) => public_key,
                    Err(e) => {
                        return Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, 
                                                  format!("failed export public key with error {}", e.to_string())));
                    }
                };
                assert!(spki_der.len() <= len);

                let buf = code.serialize(buf)?;
                let buf = {
                    if buf.len() < len {
                        Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"))
                    } else {
                        unsafe {
                            std::ptr::copy(spki_der.as_ptr(), buf.as_mut_ptr(), spki_der.len());
                        };

                        Ok(&mut buf[len..])
                        // let buf = { buf.copy_from_slice(spki_der.as_slice()); &mut buf[len..] };
                        // Ok(buf)
                    }
                }?;
                // let buf = spki_der.as_slice().serialize(buf)?;

                Ok(buf)
            }
        }
    }

}

impl Deserialize for PublicKey {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (code, buf) = u8::deserialize(buf)?;

        let len = 
            match code {
                RSA1024_PUBLIC_CODE => {
                    if buf.len() < RSA1024_PUBLIC_LENGHT_DEFAULT_MAX {
                        Err(NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_INVALID_PUBKEY, format!("invalid rsa public key length, except={}, valid rang=[0; {}]", buf.len(), RSA1024_PUBLIC_LENGHT_DEFAULT_MAX)))
                    } else {
                        Ok(RSA1024_PUBLIC_LENGHT_DEFAULT_MAX)
                    }
                }
                RSA2048_PUBLIC_CODE => {
                    if buf.len() < RSA2048_PUBLIC_LENGHT_DEFAULT_MAX {
                        Err(NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_INVALID_PUBKEY, format!("invalid rsa public key length, except={}, valid rang=[0; {}]", buf.len(), RSA2048_PUBLIC_LENGHT_DEFAULT_MAX)))
                    } else {
                        Ok(RSA2048_PUBLIC_LENGHT_DEFAULT_MAX)
                    }
                }
                RSA3072_PUBLIC_CODE => {
                    if buf.len() < RSA3072_PUBLIC_LENGHT_DEFAULT_MAX {
                        Err(NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_INVALID_PUBKEY, format!("invalid rsa public key length, except={}, valid rang=[0; {}]", buf.len(), RSA3072_PUBLIC_LENGHT_DEFAULT_MAX)))
                    } else {
                        Ok(RSA3072_PUBLIC_LENGHT_DEFAULT_MAX)
                    }
                }
                _ => { Err(NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_INVALID_PUBKEY, format!("invalid rsa public key code, except={}", code))) }
            }?;

        let (v, buf) = {
            let mut v = vec![0u8; len];
            unsafe {
                std::ptr::copy(buf.as_ptr(), v.as_mut_ptr(), len);
            }
            (v, &buf[len..])
        };

        let pk = match rsa::RSAPublicKey::from_pkcs1(v.as_slice()) {
            Ok(pk) => pk,
            Err(e) => { return Err(NearError::new(ErrorCode::NEAR_ERROR_CRYPTO_INVALID_PUBKEY, 
                                                         format!("failed import public key with error {}", e.to_string())));
            }
        };

        Ok((Self::Rsa(pk), buf))
    }
}

impl std::fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v = vec![0u8; self.raw_capacity()];

        match self.serialize(&mut v) {
            Ok(_) => {
                write!(f, "{}", hex::encode(v.as_slice()))
            }
            Err(_) => {
                write!(f, "publick invalid data")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use crate::public_key::PublicKey;

    #[test]
    fn test_public() {

        let prikey = crate::crypto_module::private_key::PrivateKey::generate_rsa1024().unwrap();
        let pubkey = prikey.public();

        let mut b = vec![0u8; pubkey.raw_capacity()];

        let _ = pubkey.serialize(&mut b).unwrap();

        println!("{:?}", b);

        let (_pubkey_2, _) = PublicKey::deserialize(&b).unwrap();

        // println!("{:?}", pubkey_2);

    }
}
