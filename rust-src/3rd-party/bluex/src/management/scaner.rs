
use log::{error, trace, debug};
use near_base::{NearResult, ErrorCode, NearError};

use crate::{bindgen::*, Config};

use super::stream::Stream;


pub struct ScanParameter {
    passive: u8,
    interval: u16,
}

impl std::default::Default for ScanParameter {
    fn default() -> Self {
        Self {
            passive: 1,
            interval: 0x30,
        }
    }
}

impl std::fmt::Display for ScanParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "passive: {}, interval: {}", self.passive, self.interval)
    }
}

impl ScanParameter {
    pub fn cmd(&self, stream: &Stream) -> NearResult<()> {
        trace!("{}", self);

        if unsafe {
            hci_le_set_scan_parameters(stream.dd(),
                                       self.passive,
                                       self.interval,
                                       self.interval,
                                       0,
                                       0,
                                       Config::get_instace().timeout_interval as i32)
        } < 0 {
            let error_string = format!("failed scan param.");
            error!("{error_string}");
            Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string))
        } else {
            Ok(())
        }
    }
}

pub struct ScanSwitch {
    switch: u8,
}

impl std::fmt::Display for ScanSwitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if self.switch == 0 { "closed" } else { "open"} )
    }
}

impl ScanSwitch {
    pub fn open_scan() -> Self {
        Self{
            switch: 1,
        }
    }

    pub fn close_scan() -> Self {
        Self {
            switch: 0,
        }
    }

    pub fn cmd(&self, stream: &Stream) -> NearResult<()> {
        trace!("{}", self);

        if unsafe {
            hci_le_set_scan_enable(stream.dd(),
                                    self.switch,
                                    1,
                                    Config::get_instace().timeout_interval as i32)
        } < 0 {
            let error_string = format!("failed scan switch.");
            error!("{error_string}");
            Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string))
        } else {
            Ok(())
        }
    }
}

pub struct Scanning {
    stream: Stream,
    of: hci_filter,
}

pub struct ScanResult {
    pub addr: mac_address::MacAddress,
    // pub addr: [u8; 6],
    pub data: Vec<u8>,
}

impl Scanning {
    pub fn open(stream: Stream) -> NearResult<Self> {
        let of = stream.getsockopt::<hci_filter>(SOL_HCI as i32, HCI_FILTER as i32)?;

        debug!("get original filter");
        let mut nf: ::std::mem::MaybeUninit<hci_filter> = ::std::mem::MaybeUninit::uninit();
        let nf_ptr = nf.as_mut_ptr();

        debug!("set newest filter");
        unsafe {
            hci_filter_clear(nf_ptr);
            hci_filter_set_ptype(HCI_EVENT_PKT as i32, nf_ptr);
            hci_filter_set_event(EVT_LE_META_EVENT as i32, nf_ptr);
        }
        stream.setsockopt(SOL_HCI as i32, HCI_FILTER as i32, &nf)?;

        Ok(Self { stream, of })
    } 
}

impl std::ops::Drop for Scanning {
    fn drop(&mut self) {
        debug!("reset filter");
        let _ = self.stream.setsockopt(SOL_HCI as i32, HCI_FILTER as i32, &self.of);        
    }
}

impl Scanning {
    pub async fn scanning(&mut self) -> NearResult<ScanResult> {

        let mut data = vec![0u8; HCI_MAX_EVENT_SIZE as usize];

        match self.stream.recv(&mut data) {
            Ok(len) if len > 0 => {
                let mut raddr = std::mem::MaybeUninit::<bdaddr_t>::uninit();
                let mut rdata = std::mem::MaybeUninit::<le_set_scan_response_data_cp>::uninit();

                match unsafe {
                    parse_advertising_info(data.as_ptr(), 
                                            len as u16,
                                            raddr.as_mut_ptr(),
                                            rdata.as_mut_ptr()) 
                } {
                    0 => {
                        debug!("successful parse advertising data.");

                        let addr = {
                            unsafe {
                                let mut b = raddr.assume_init_mut().b;
                                // b.reverse();
                                // b
                                b.reverse();
                                mac_address::MacAddress::new(b)
                            }
                        };

                        let data = unsafe { *rdata.as_ptr() };

                        Ok(ScanResult { addr, data: data.data.to_vec() })
                    }
                    _ => Err(NearError::new(ErrorCode::NEAR_ERROR_RETRY, "retry"))
                }
            }
            Err(e) => Err(e),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_RETRY, "retry"))

        }
    }
}
