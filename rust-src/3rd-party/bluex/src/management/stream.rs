
use std::{mem::{MaybeUninit, size_of}, os::{unix::net::UnixStream, fd::{FromRawFd, AsRawFd, RawFd}}, io::{Read, ErrorKind}};

use enumflags2::{bitflags, BitFlags};
use libc::{c_int, socklen_t};
use log::{error, trace};

use near_base::{NearResult, NearError, ErrorCode};

use crate::bindgen::{hci_close_dev, hci_get_route, hci_open_dev, set_sock_non_block, set_sock_recv_size};

#[repr(u32)]
#[bitflags]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StreamFlags {
    NonBlock = 1 << 0,
}

pub struct Stream {
    handle: UnixStream,
}

impl Clone for Stream {
    fn clone(&self) -> Self {
        Self {
            handle: unsafe { UnixStream::from_raw_fd(self.dd()) }
        }
    }
}

impl std::ops::Drop for Stream {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl Stream {
    pub fn open(dev_id: i32, flags: BitFlags<StreamFlags>) -> NearResult<Self> {
        let handle = unsafe { hci_open_dev(dev_id) };

        if handle < 0 {
            let error_string = format!("failed open device {dev_id}.");
            error!("{error_string}");
            Err(NearError::new(ErrorCode::NEAR_ERROR_3RD, error_string))
        } else {

            let r = Self{
                handle: unsafe { UnixStream::from_raw_fd(handle) },
            };

            if flags.contains(StreamFlags::NonBlock) {
                unsafe { set_sock_non_block(r.dd()) };
            }

            unsafe {
                set_sock_recv_size(r.dd(), 4*1024);
            }

            Ok(r)
        }
    }

    pub fn open_default(flags: BitFlags<StreamFlags>) -> NearResult<Self> {
        let device_id = unsafe {
            hci_get_route(std::ptr::null_mut())
        };

        if device_id < 0 {
            let error_string = format!("failed get device.");
            error!("{error_string}");
            Err(NearError::new(ErrorCode::NEAR_ERROR_3RD, error_string))
        } else {
            Ok(Self::open(device_id, flags)?)
        }
    }

    pub(self) fn close(&self) -> NearResult<()> {
        trace!("close {}", self.dd());

        if unsafe { hci_close_dev(self.dd()) } < 0 {
            Err(NearError::new(ErrorCode::NEAR_ERROR_3RD, format!("failed close {}", self.dd())))
        } else {
            Ok(())
        }
    }
    
}

impl Stream {
    /// Get socket option.
    pub fn getsockopt<T>(&self, level: c_int, optname: c_int) -> NearResult<T> {
        let mut optval: MaybeUninit<T> = MaybeUninit::uninit();
        let mut optlen: socklen_t = size_of::<T>() as _;
        if unsafe { libc::getsockopt(self.dd(), level, optname, optval.as_mut_ptr() as *mut _, &mut optlen) }
            == -1
        {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, format!("{}", std::io::Error::last_os_error())));
        }

        if optlen != size_of::<T>() as _ {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "invalide size"));
        }

        let optval = unsafe { optval.assume_init() };
        Ok(optval)
    }

    /// Set socket option.
    pub fn setsockopt<T>(&self, level: c_int, optname: i32, optval: &T) -> NearResult<()> {
        let optlen: socklen_t = size_of::<T>() as _;
        if unsafe { libc::setsockopt(self.dd(), level, optname, optval as *const _ as *const _, optlen) }
            == -1
        {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, format!("{}", std::io::Error::last_os_error())));
        }
        Ok(())
}

}

impl Stream {
    pub fn dd(&self) -> RawFd {
        self.handle.as_raw_fd()
    }

    pub fn recv(&mut self, data: &mut Vec<u8>) -> NearResult<usize> {
        self.handle.read(data)
            .map_err(| e | {
                match e.kind() {
                    ErrorKind::WouldBlock => NearError::new(ErrorCode::NEAR_ERROR_RETRY, "retry it."),
                    _ => NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, "exception"),
                }
            })
    }
}

