
use crate::{Serialize, Deserialize, };


#[allow(non_camel_case_types)]
#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq, )]
pub enum ErrorCode {
    NEAR_ERROR_SUCCESS                    = 0,
    // 1~10 系统级错误
    NEAR_ERROR_FATAL                      = 1,
    NEAR_ERROR_SYSTERM                    = 2,
    NEAR_ERROR_3RD                        = 3,
    NEAR_COMMAND_MAJOR                    = 4,
    NEAR_COMMAND_MINOR                    = 5,
    NEAR_ERROR_EXCEPTION                  = 6,
    NEAR_ERROR_NOTFOUND                   = 7,
    NEAR_ERROR_NO_AVAILABLE               = 8,
    NEAR_ERROR_RETRY                      = 9,
    NEAR_ERROR_STARTUP                    = 10,
    NEAR_ERROR_NO_TARGET                  = 11,
    NEAR_ERROR_ALREADY_EXIST              = 12,
    NEAR_ERROR_DONOT_EXIST                = 13,
    NEAR_ERROR_UNDEFINED                  = 14,
    NEAR_ERROR_CONFLICT                   = 15,
    NEAR_ERROR_FORBIDDEN                  = 16,
    NEAR_ERROR_EXPIRED                    = 17,
    NEAR_ERROR_MISSING_DATA               = 18,
    NEAR_ERROR_UNMATCH                    = 19,
    NEAR_ERROR_INCORRECT_USE              = 20,

    // 21~100底层错误
    NEAR_ERROR_INVALIDPARAM               = 21,
    NEAR_ERROR_OUTOFLIMIT                 = 22,
    NEAR_ERROR_INVALIDFORMAT              = 23,
    NEAR_ERROR_UNKNOWN_PROTOCOL           = 24,
    NEAR_ERROR_INVALID_ADDRSOCKET         = 25,
    NEAR_ERROR_PROTOCOL_NEED_EXCHANGE     = 26,
    NEAR_ERROR_TUNNEL_CLOSED              = 27,
    NEAR_ERROR_REFUSE                     = 28,
    NEAR_ERROR_UNINITIALIZED              = 29,
    NEAR_ERROR_UNACTIVED                  = 30,
    NEAR_ERROR_ACTIVED                    = 31,
    NEAR_ERROR_TIMEOUT                    = 32,
    NEAR_ERROR_IGNORE                     = 33,
    NEAR_ERROR_STATE                      = 34,
    NEAR_ERROR_OUTOFMEMORY                = 35,
    NEAR_ERROR_NOT_ENOUGH                 = 36,
    NEAR_ERROR_OPERATOR_COMPLETED         = 37,
    NEAR_ERROR_ENCODING_FORMAT            = 38,

    NEAR_ERROR_REFERER                    = 39,
    NEAR_ERROR_DISSUPPORT                 = 40,

    NEAR_ERROR_CRYPTO_GENRSA              = 41,
    NEAR_ERROR_CRYPTO_ENCRYPT             = 42,
    NEAR_ERROR_CRYPTO_DECRYPT             = 43,
    NEAR_ERROR_CRYPTO_SIGN                = 44,
    NEAR_ERROR_CRYPTO_VERIFY              = 45,
    NEAR_ERROR_CRYPTO_SIGNDATA_OUTOFLIMIT = 46,
    NEAR_ERROR_CRYPTO_INVALID_PUBKEY      = 47,

    NEAR_ERROR_CRYPTO_AEK_ENCRYPT         = 51,
    NEAR_ERROR_CRYPTO_AEK_DECRYPT         = 52,

    // TOPIC
    NEAR_ERROR_TOPIC_EXCEPTION            = 61,
    NEAR_ERROR_TOPIC_ROOT                 = 62,
    NEAR_ERROR_TOPIC_PRIMARY              = 63,
    NEAR_ERROR_TOPIC_SECONDARY            = 64,
    NEAR_ERROR_TOPIC_UNKNOWN              = 69,

    // protobuf
    NEAR_ERROR_PROTOC_ENCODE              = 70,
    NEAR_ERROR_PROTOC_DECODE              = 71,
    NEAR_ERROR_PROTOC_SET_FIELD           = 72,
    NEAR_ERROR_PROTOC_NOT_FIELD           = 73,
    NEAR_ERROR_PROTOC_NOT_MESSAGE         = 74,
    NEAR_ERROR_PROTOC_NOT_SUPPORT_MAP     = 75,
    NEAR_ERROR_PROTOC_NOT_SUPPORT_REPEATED= 76,

    // dataagent
    NEAR_ERROR_DATAAGNT_NEED_BIND_PARAMS  = 90,

    NEAR_ERROR_UNKNOWN                    = 255,

}

impl std::default::Default for ErrorCode {
    fn default() -> Self {
        Self::NEAR_ERROR_SUCCESS
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v: u16 = self.into_u16();
        write!(f, "{}", v)
    }
}

impl ErrorCode {
    pub fn into_u16(&self) -> u16 {
        match self {
            ErrorCode::NEAR_ERROR_SUCCESS                   	=> 0,
            ErrorCode::NEAR_ERROR_FATAL                     	=> 1,
            ErrorCode::NEAR_ERROR_SYSTERM                   	=> 2,
            ErrorCode::NEAR_ERROR_3RD                       	=> 3,
            ErrorCode::NEAR_COMMAND_MAJOR                   	=> 4,
            ErrorCode::NEAR_COMMAND_MINOR                   	=> 5,
            ErrorCode::NEAR_ERROR_EXCEPTION                 	=> 6,
            ErrorCode::NEAR_ERROR_NOTFOUND                  	=> 7,
            ErrorCode::NEAR_ERROR_NO_AVAILABLE              	=> 8,
            ErrorCode::NEAR_ERROR_RETRY                     	=> 9,
            ErrorCode::NEAR_ERROR_STARTUP                       => 10,
            ErrorCode::NEAR_ERROR_NO_TARGET                     => 11,
            ErrorCode::NEAR_ERROR_ALREADY_EXIST                 => 12,
            ErrorCode::NEAR_ERROR_DONOT_EXIST                   => 13,
            ErrorCode::NEAR_ERROR_UNDEFINED                     => 14,
            ErrorCode::NEAR_ERROR_CONFLICT                      => 15,
            ErrorCode::NEAR_ERROR_FORBIDDEN                     => 16,
            ErrorCode::NEAR_ERROR_EXPIRED                       => 17,
            ErrorCode::NEAR_ERROR_MISSING_DATA                  => 18,
            ErrorCode::NEAR_ERROR_UNMATCH                       => 19,
            ErrorCode::NEAR_ERROR_INCORRECT_USE                 => 20,
            ErrorCode::NEAR_ERROR_INVALIDPARAM              	=> 21,
            ErrorCode::NEAR_ERROR_OUTOFLIMIT                	=> 22,
            ErrorCode::NEAR_ERROR_INVALIDFORMAT             	=> 23,
            ErrorCode::NEAR_ERROR_UNKNOWN_PROTOCOL          	=> 24,
            ErrorCode::NEAR_ERROR_INVALID_ADDRSOCKET        	=> 25,
            ErrorCode::NEAR_ERROR_PROTOCOL_NEED_EXCHANGE    	=> 26,
            ErrorCode::NEAR_ERROR_TUNNEL_CLOSED             	=> 27,
            ErrorCode::NEAR_ERROR_REFUSE                    	=> 28,
            ErrorCode::NEAR_ERROR_UNINITIALIZED             	=> 29,
            ErrorCode::NEAR_ERROR_UNACTIVED                 	=> 30,
            ErrorCode::NEAR_ERROR_ACTIVED                   	=> 31,
            ErrorCode::NEAR_ERROR_TIMEOUT                   	=> 32,
            ErrorCode::NEAR_ERROR_IGNORE                    	=> 33,
            ErrorCode::NEAR_ERROR_STATE                     	=> 34,
            ErrorCode::NEAR_ERROR_OUTOFMEMORY               	=> 35,
            ErrorCode::NEAR_ERROR_NOT_ENOUGH                	=> 36,
            ErrorCode::NEAR_ERROR_OPERATOR_COMPLETED        	=> 37,
            ErrorCode::NEAR_ERROR_ENCODING_FORMAT           	=> 38,
            ErrorCode::NEAR_ERROR_REFERER                   	=> 39,
            ErrorCode::NEAR_ERROR_DISSUPPORT                    => 40,
            ErrorCode::NEAR_ERROR_CRYPTO_GENRSA             	=> 41,
            ErrorCode::NEAR_ERROR_CRYPTO_ENCRYPT            	=> 42,
            ErrorCode::NEAR_ERROR_CRYPTO_DECRYPT            	=> 43,
            ErrorCode::NEAR_ERROR_CRYPTO_SIGN               	=> 44,
            ErrorCode::NEAR_ERROR_CRYPTO_VERIFY             	=> 45,
            ErrorCode::NEAR_ERROR_CRYPTO_SIGNDATA_OUTOFLIMIT	=> 46,
            ErrorCode::NEAR_ERROR_CRYPTO_INVALID_PUBKEY     	=> 47,
            ErrorCode::NEAR_ERROR_CRYPTO_AEK_ENCRYPT        	=> 51,
            ErrorCode::NEAR_ERROR_CRYPTO_AEK_DECRYPT        	=> 52,
            ErrorCode::NEAR_ERROR_TOPIC_EXCEPTION           	=> 61,
            ErrorCode::NEAR_ERROR_TOPIC_ROOT                	=> 62,
            ErrorCode::NEAR_ERROR_TOPIC_PRIMARY             	=> 63,
            ErrorCode::NEAR_ERROR_TOPIC_SECONDARY           	=> 64,
            ErrorCode::NEAR_ERROR_TOPIC_UNKNOWN           	    => 69,
            ErrorCode::NEAR_ERROR_PROTOC_ENCODE                 => 70,
            ErrorCode::NEAR_ERROR_PROTOC_DECODE                 => 71,
            ErrorCode::NEAR_ERROR_PROTOC_SET_FIELD              => 72,
            ErrorCode::NEAR_ERROR_PROTOC_NOT_FIELD              => 73,
            ErrorCode::NEAR_ERROR_PROTOC_NOT_MESSAGE            => 74,
            ErrorCode::NEAR_ERROR_PROTOC_NOT_SUPPORT_MAP        => 75,
            ErrorCode::NEAR_ERROR_PROTOC_NOT_SUPPORT_REPEATED   => 76,
            ErrorCode::NEAR_ERROR_DATAAGNT_NEED_BIND_PARAMS     => 90,
            ErrorCode::NEAR_ERROR_UNKNOWN                   	=> 255,
   

        }
    }

}

impl From<u16> for ErrorCode {
    fn from(v: u16) -> Self {
        match v {
            0       => ErrorCode::NEAR_ERROR_SUCCESS,
            1       => ErrorCode::NEAR_ERROR_FATAL,
            2       => ErrorCode::NEAR_ERROR_SYSTERM,
            3       => ErrorCode::NEAR_ERROR_3RD,
            4       => ErrorCode::NEAR_COMMAND_MAJOR,
            5       => ErrorCode::NEAR_COMMAND_MINOR,
            6       => ErrorCode::NEAR_ERROR_EXCEPTION,
            7       => ErrorCode::NEAR_ERROR_NOTFOUND,
            8       => ErrorCode::NEAR_ERROR_NO_AVAILABLE,
            9       => ErrorCode::NEAR_ERROR_RETRY,
            10      => ErrorCode::NEAR_ERROR_STARTUP,
            11      => ErrorCode::NEAR_ERROR_NO_TARGET,
            12      => ErrorCode::NEAR_ERROR_ALREADY_EXIST,
            13      => ErrorCode::NEAR_ERROR_DONOT_EXIST,
            14      => ErrorCode::NEAR_ERROR_UNDEFINED,
            15      => ErrorCode::NEAR_ERROR_CONFLICT,
            16      => ErrorCode::NEAR_ERROR_FORBIDDEN,
            17      => ErrorCode::NEAR_ERROR_EXPIRED,
            18      => ErrorCode::NEAR_ERROR_MISSING_DATA,
            19      => ErrorCode::NEAR_ERROR_UNMATCH,
            20      => ErrorCode::NEAR_ERROR_INCORRECT_USE,
            21      => ErrorCode::NEAR_ERROR_INVALIDPARAM,
            22      => ErrorCode::NEAR_ERROR_OUTOFLIMIT,
            23      => ErrorCode::NEAR_ERROR_INVALIDFORMAT,
            24      => ErrorCode::NEAR_ERROR_UNKNOWN_PROTOCOL,
            25      => ErrorCode::NEAR_ERROR_INVALID_ADDRSOCKET,
            26      => ErrorCode::NEAR_ERROR_PROTOCOL_NEED_EXCHANGE,
            27      => ErrorCode::NEAR_ERROR_TUNNEL_CLOSED,
            28      => ErrorCode::NEAR_ERROR_REFUSE,
            29      => ErrorCode::NEAR_ERROR_UNINITIALIZED,
            30      => ErrorCode::NEAR_ERROR_UNACTIVED,
            31      => ErrorCode::NEAR_ERROR_ACTIVED,
            32      => ErrorCode::NEAR_ERROR_TIMEOUT,
            33      => ErrorCode::NEAR_ERROR_IGNORE,
            34      => ErrorCode::NEAR_ERROR_STATE,
            35      => ErrorCode::NEAR_ERROR_OUTOFMEMORY,
            36      => ErrorCode::NEAR_ERROR_NOT_ENOUGH,
            37      => ErrorCode::NEAR_ERROR_OPERATOR_COMPLETED,
            38      => ErrorCode::NEAR_ERROR_ENCODING_FORMAT,
            39      => ErrorCode::NEAR_ERROR_REFERER,
            40      => ErrorCode::NEAR_ERROR_DISSUPPORT,
            41      => ErrorCode::NEAR_ERROR_CRYPTO_GENRSA,
            42      => ErrorCode::NEAR_ERROR_CRYPTO_ENCRYPT,
            43      => ErrorCode::NEAR_ERROR_CRYPTO_DECRYPT,
            44      => ErrorCode::NEAR_ERROR_CRYPTO_SIGN,
            45      => ErrorCode::NEAR_ERROR_CRYPTO_VERIFY,
            46      => ErrorCode::NEAR_ERROR_CRYPTO_SIGNDATA_OUTOFLIMIT,
            47      => ErrorCode::NEAR_ERROR_CRYPTO_INVALID_PUBKEY,
            51      => ErrorCode::NEAR_ERROR_CRYPTO_AEK_ENCRYPT,
            52      => ErrorCode::NEAR_ERROR_CRYPTO_AEK_DECRYPT,
            61      => ErrorCode::NEAR_ERROR_TOPIC_EXCEPTION,
            62      => ErrorCode::NEAR_ERROR_TOPIC_ROOT,
            63      => ErrorCode::NEAR_ERROR_TOPIC_PRIMARY,
            64      => ErrorCode::NEAR_ERROR_TOPIC_SECONDARY,
            69      => ErrorCode::NEAR_ERROR_TOPIC_UNKNOWN,
            70      => ErrorCode::NEAR_ERROR_PROTOC_ENCODE,
            71      => ErrorCode::NEAR_ERROR_PROTOC_DECODE,
            72      => ErrorCode::NEAR_ERROR_PROTOC_SET_FIELD,
            73      => ErrorCode::NEAR_ERROR_PROTOC_NOT_FIELD,
            74      => ErrorCode::NEAR_ERROR_PROTOC_NOT_MESSAGE,
            75      => ErrorCode::NEAR_ERROR_PROTOC_NOT_SUPPORT_MAP,
            76      => ErrorCode::NEAR_ERROR_PROTOC_NOT_SUPPORT_REPEATED,
            90      => ErrorCode::NEAR_ERROR_DATAAGNT_NEED_BIND_PARAMS,
            255 | _ => ErrorCode::NEAR_ERROR_UNKNOWN,
        }
    }
}

#[derive(Clone)]
pub struct NearError {
    errno: ErrorCode,
    error_message: Option<String>,
}

pub type NearResult<T> = Result<T, NearError>;

impl std::default::Default for NearError {
    fn default() -> Self {
        Self {
            errno: ErrorCode::NEAR_ERROR_SUCCESS,
            error_message: None,
        }
    }
}

impl NearError {
    pub fn new(code: ErrorCode, message: impl std::string::ToString) -> Self {
        Self {
            errno: code,
            error_message: Some(message.to_string()),
        }
    }

    pub fn errno(&self) -> ErrorCode {
        self.errno
    }

    pub fn into_errno(&self) -> u16 {
        self.errno().into_u16()
    }

    pub fn error_message<'a>(&'a self) -> Option<&'a str> {
        self.error_message
            .as_ref()
            .map(| message | message.as_str())
    }

    pub fn split(self) -> (ErrorCode, Option<String>) {
        (self.errno, self.error_message)
    }

}

impl From<std::io::Error> for NearError {
    fn from(err: std::io::Error) -> NearError {
        NearError::new(ErrorCode::NEAR_ERROR_SYSTERM,
                       format!("failed operator with errno={}", err))
    }
}

impl std::fmt::Debug for NearError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.errno {
            ErrorCode::NEAR_ERROR_SUCCESS => {
                write!(f, "errno={}", self.errno)
            }
            _ => {
                write!(f, "errno={}, error_message={}", self.errno, self.error_message().unwrap())
            }
        }
    }

}

impl std::fmt::Display for NearError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn std::fmt::Debug).fmt(f)
    }
}

impl Serialize for NearError {
    fn raw_capacity(&self) -> usize {
        0u16.raw_capacity() + self.error_message.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.into_errno().serialize(buf)?;
        let buf = self.error_message.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for NearError {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (errno, buf) = u16::deserialize(buf)
                            .map(| (errno, buf) |{
                                (errno.into(), buf)
                            })?;
        let (error_message, buf) = Option::<String>::deserialize(buf)?;

        Ok((Self{
            errno,
            error_message
        }, buf))
    }
}

// impl<T: Serialize + Deserialize> Serialize for NearResult<T> {
//     fn raw_capacity(&self) -> usize {
//         match self {
//             Ok(data) => {
//                 0u16.raw_capacity() + data.raw_capacity()
//             }
//             Err(err) => {
//                 err.into_errno().raw_capacity() + err.error_message.raw_capacity()
//             }
//         }
//     }

//     fn serialize<'a>(&self,
//                      buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
//         match self {
//             Ok(data) => {
//                 let buf = 0u16.serialize(buf)?;
//                 let buf = data.serialize(buf)?;

//                 Ok(buf)
//             }
//             Err(err) => {
//                 let buf = err.into_errno().serialize(buf)?;
//                 let buf = err.error_message.serialize(buf)?;

//                 Ok(buf)
//             }
//         }
//     }

// }

// impl<T: Serialize + Deserialize> Deserialize for NearResult<T> {
//     fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
//         let (errno, buf) = u16::deserialize(buf)
//                             .map(| (errno, buf) |{
//                                 (errno.into(), buf)
//                             })?;

//         match &errno {
//             ErrorCode::NEAR_ERROR_SUCCESS => {
//                 let (data, buf) = T::deserialize(buf)?;
//                 Ok((Ok(data), buf))
//             }
//             _ => {
//                 let (message, buf) = String::deserialize(buf)?;
//                 Ok((Err(NearError::new(errno, message)), buf))
//             }
//         }
//     }
// }

#[test]
fn test_error() {

    use base58::ToBase58;

    let e = NearError::default();
    // let e = NearError::new(11u16, "abc");

    let mut v = vec![0u8; e.raw_capacity()];
    e.serialize(v.as_mut_slice()).unwrap();
    println!("{}", v.to_base58());

    // let msg = dsg_err_msg!(123);

    println!("code={}, message={:?}", e.into_errno(), e.error_message());
}
