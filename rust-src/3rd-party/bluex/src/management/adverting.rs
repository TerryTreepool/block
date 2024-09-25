
use log::{trace, error};
use near_base::{NearResult, NearError, ErrorCode};

use crate::{bindgen::{hci_le_set_advertising_paramters, hci_le_enable_advertising, hci_le_advertising_data, LE_SET_SCAN_RESPONSE_DATA_CP_SIZE}, Config};

use super::stream::Stream;


pub struct AdvertisingParam {
    min_interval: u16,
    max_interval: u16,
}

impl std::fmt::Display for AdvertisingParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "min_interval: {}, max_interval: {}", self.min_interval, self.max_interval)
    }
}

impl AdvertisingParam {
    pub fn new(min_interval: u16, max_interval: u16) -> Self {
        Self {
            min_interval,
            max_interval,
        }
    }

    pub fn cmd(&self, stream: &Stream) -> NearResult<()> {
        trace!("{}", self);

        if unsafe {
            hci_le_set_advertising_paramters(stream.dd(),
                                            self.min_interval.into(),
                                            self.max_interval.into(),
                                            Config::get_instace().timeout_interval)
        } < 0 {
            let error_string = format!("failed advertising param.");
            error!("{error_string}");
            Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string))
        } else {
            Ok(())
        }
    }
}

pub struct AdvertisingSwitch {
    switch: u8,
}

impl std::fmt::Display for AdvertisingSwitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.switch {
            0 => write!(f, "disabled advertising."),
            _ => write!(f, "enabled advertising.")
        }
    }
}

impl AdvertisingSwitch {
    pub fn open_advertising() -> Self {
        Self {
            switch: 1,
        }
    }

    pub fn close_advertising() -> Self {
        Self {
            switch: 0,
        }
    }

    pub fn cmd(&self, stream: &Stream) -> NearResult<()> {
        trace!("{}", self);

        if unsafe {
            hci_le_enable_advertising(stream.dd(),
                                      if self.switch == 0 { 0 } else { 1 },
                                      Config::get_instace().timeout_interval)
        } < 0 {
            let error_string = format!("failed advertising switch.");
            error!("{error_string}");
            Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string))
        } else {
            Ok(())
        }
    }
}

pub struct AdvertisingData {
    data: Vec<u8>,
}

impl std::fmt::Display for AdvertisingData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Advertising Data: [{}]", hex::encode_upper(&self.data))
    }
}

impl AdvertisingData {
    pub fn new(data: Vec<u8>) -> NearResult<Self> {
        if data.len() >= LE_SET_SCAN_RESPONSE_DATA_CP_SIZE as usize {
            Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, format!("data-len out limit, max is {}", LE_SET_SCAN_RESPONSE_DATA_CP_SIZE - 1)))
        } else {
            Ok(Self{
                data
            })
        }
    }

    pub fn cmd(&self, stream: &Stream) -> NearResult<()> {
        trace!("{}", self);

        if unsafe {
            hci_le_advertising_data(stream.dd(),
                                    self.data.as_ptr(),
                                    self.data.len() as u8,
                                    Config::get_instace().timeout_interval)
        } < 0 {
            let error_string = format!("failed advertising data.");
            error!("{error_string}");
            Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string))
        } else {
            Ok(())
        }
    }
}

