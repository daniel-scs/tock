//! CRC driver

use core::cell::Cell;
use kernel::{AppId, Callback, Driver, ReturnCode};
use kernel::hil;

pub struct Crc<'a, C: hil::crc::CRC + 'a> {
    crc_unit: &'a C,
    callback: Cell<Option<Callback>>,
}

impl<'a, C: hil::crc::CRC> Crc<'a, C> {
    pub fn new(crc_unit: &'a C) -> Crc<'a, C> {
        Crc { crc_unit: crc_unit,
              callback: Cell::new(None),
            }
    }
}

static CRC_TEST_DATA: &'static [u8] = b"ABCDEFG";

impl<'a, C: hil::crc::CRC> Driver for Crc<'a, C>  {
    fn command(&self, command_num: usize, _data: usize, _appid: AppId) -> ReturnCode {
        match command_num {
            // The driver is present
            0 => ReturnCode::SUCCESS,

            // Get version
            1 => ReturnCode::SuccessWithValue{ value: self.crc_unit.get_version() as usize },

            // Initialize unit
            2 => self.crc_unit.init(),

            // Request computation
            3 => self.crc_unit.compute(CRC_TEST_DATA),

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // Set callback for CRC result
            0 => {
                self.callback.set(Some(callback));
                ReturnCode::SUCCESS
            },

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a, C: hil::crc::CRC> hil::crc::Client for Crc<'a, C> {
    fn receive_result(&self, result: u32) {
        if let Some(mut callback) = self.callback.get() {
            callback.schedule(result as usize, 0, 0);
        }
    }
}
