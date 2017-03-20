//! CRC driver

use kernel::{AppId, Driver, ReturnCode};
use kernel::hil;

pub struct Crc<'a, C: hil::CRC + 'a> {
    crc_unit: &'a C,
}

impl<'a, C: hil::CRC> Crc<'a, C> {
    pub fn new(crc_unit: C) {
        Crc{ crc_unit: crc_unit }
    }
}

impl<'a, C: hil::CRC> Driver for Crc<'a, C>  {
    fn command(&self, command_num: usize, data: usize, appid: AppId) -> ReturnCode {
        match command_num {
            // The driver is present
            0 => ReturnCode::Success,

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
