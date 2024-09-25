
use crate::ErrorCode;
use crate::errors::{NearError, NearResult};
use crate::codec::{Serialize, Deserialize, RawFixedBytes};

const RSA1024_LENGHT_DEFAULT_MAX: usize = 128;
const RSA2048_LENGHT_DEFAULT_MAX: usize = 256;

#[derive(Clone)]
pub enum SignData {
    Rsa1024([u8; RSA1024_LENGHT_DEFAULT_MAX]),
    Rsa2048([u8; RSA2048_LENGHT_DEFAULT_MAX]),
}

impl SignData {
    pub fn as_slice(&self) -> &[u8] {
        match self {
            SignData::Rsa1024(v) => {
                v
            }
            SignData::Rsa2048(v) => {
                v
            }
        }
    }
}

impl TryFrom<&[u8]> for SignData {
    type Error = NearError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value.len() {
            RSA1024_LENGHT_DEFAULT_MAX => {
                let mut v = [0u8; RSA1024_LENGHT_DEFAULT_MAX];

                v.copy_from_slice(value);

                return Ok(SignData::Rsa1024(v));
            }
            RSA2048_LENGHT_DEFAULT_MAX => {
                let mut v = [0u8; RSA2048_LENGHT_DEFAULT_MAX];

                v.copy_from_slice(value);

                return Ok(SignData::Rsa2048(v));
            }
            _ => {
                // warn!("failed signature data with signature data length not enough, except={}, get=256", value.len());
                unreachable!()
            }
        }
    }
}

impl Serialize for SignData {
    fn raw_capacity(&self) -> usize {
        let len = 
            match self {
                Self::Rsa1024(_) => RSA1024_LENGHT_DEFAULT_MAX,
                Self::Rsa2048(_) => RSA2048_LENGHT_DEFAULT_MAX,
            };

        len + u8::raw_bytes()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {

        let (code, capacity) = 
            match self {
                Self::Rsa1024(_) => (1u8, RSA1024_LENGHT_DEFAULT_MAX),
                Self::Rsa2048(_) => (2u8, RSA2048_LENGHT_DEFAULT_MAX),
            };

        let buf = code.serialize(buf)?;
        let buf = {
            if buf.len() < capacity {
                Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"))
            } else {
                buf[..capacity].copy_from_slice(self.as_slice());
                Ok(&mut buf[capacity..])
            }
        }?;

        Ok(buf)
    }
}

impl Deserialize for SignData {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (code, buf) = u8::deserialize(buf)?;

        let len = 
            match code {
                1u8 => { Ok(RSA1024_LENGHT_DEFAULT_MAX) }
                2u8 => { Ok(RSA2048_LENGHT_DEFAULT_MAX) }
                _ => { Err(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN, format!("{} unknown code.", code))) }
            }?;

        if buf.len() < len {
            Err(NearError::new(ErrorCode::NEAR_ERROR_NOT_ENOUGH, "not enough buffer"))
        } else {
            Ok(())
        }?;

        let (v, buf) =  {
            let mut v = vec![0u8; len];
            unsafe {
                std::ptr::copy(buf.as_ptr(), v.as_mut_ptr(), len);
            }
            (v, &buf[len..])
        };

        let r = {
            match v.len() {
                RSA1024_LENGHT_DEFAULT_MAX => {
                    let mut val = [0u8; RSA1024_LENGHT_DEFAULT_MAX];
                    val.copy_from_slice(v.as_slice());
                    (Self::Rsa1024(val), buf)
                }
                RSA2048_LENGHT_DEFAULT_MAX => {
                    let mut val = [0u8; RSA2048_LENGHT_DEFAULT_MAX];
                    val.copy_from_slice(v.as_slice());
                    (Self::Rsa2048(val), buf)
                }
                _ => {
                    // warn!("failed signature data with signature data length not enough, except={}, get=256", value.len());
                    unreachable!()
                }
            }
        };

        Ok(r)

    }

}

#[derive(Clone)]
pub struct Signature {
    sign_time: u64,
    sign_data: SignData,
}

impl Signature {
    pub fn new(sign_time: u64, sign_data: SignData) -> Self {
        Self { sign_time, sign_data }
    }

    pub fn sign_time(&self) -> u64 {
        self.sign_time
    }

    pub fn sign_data(&self) -> &SignData {
        &self.sign_data
    }

    pub fn as_slice(&self) -> &[u8] {
        self.sign_data.as_slice()
    }
}

impl Serialize for Signature {
    fn raw_capacity(&self) -> usize {
        self.sign_time.raw_capacity() + self.sign_data.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.sign_time.serialize(buf)?;

        let buf = self.sign_data.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for Signature {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (sign_time, buf) = u64::deserialize(buf)?;

        let (sign_data, buf) = SignData::deserialize(buf)?;

        Ok((Self{
            sign_time, sign_data
        }, buf))
    }

}

