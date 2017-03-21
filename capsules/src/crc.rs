//! CRC driver

use kernel::{AppId, Driver, ReturnCode};
use kernel::hil;

pub struct Crc<'a, C: hil::crc::CRC + 'a> {
    crc_unit: &'a C,
}

impl<'a, C: hil::crc::CRC> Crc<'a, C> {
    pub fn new(crc_unit: &'a C) -> Crc<'a, C> {
        Crc{ crc_unit: crc_unit }
    }
}

impl<'a, C: hil::crc::CRC> Driver for Crc<'a, C>  {
    fn command(&self, command_num: usize, _data: usize, _appid: AppId) -> ReturnCode {
        match command_num {
            // The driver is present
            0 => ReturnCode::SUCCESS,

            1 => ReturnCode::SuccessWithValue{ value: self.crc_unit.get_version() as usize },

            2 => self.crc_unit.init(),

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // Set callback for CRC result
            0 => {
                self.callback
                    .enter(callback.app_id(), |cntr, _| {
                        cntr.0 = Some(callback);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            },

            _ => ReturnCode::ENOSUPPORT,
        }
    }

}
