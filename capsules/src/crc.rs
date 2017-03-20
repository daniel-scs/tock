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

            1 => {
                if self.crc_unit.get_version() == 0x00000202 {
                    ReturnCode::SUCCESS
                }
                else {
                    ReturnCode::FAIL
                }
            }


            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
