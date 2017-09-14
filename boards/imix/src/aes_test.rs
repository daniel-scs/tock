//! Test the AES hardware

use core::cell::Cell;
use kernel::ReturnCode;
use kernel::hil;
use kernel::hil::symmetric_encryption::{AES128_BLOCK_SIZE, AES128, AES128Ctr};
use kernel::common::{List, ListLink, ListNode};
use kernel::common::take_cell::{TakeCell};
use kernel::common::async_take_cell::*;

use kernel::aes_test;
use sam4l;

// A cell holding a reference to the sam4l's AES hardware
static AES_CELL: AsyncTakeCell<'static, sam4l::aes::Aes> = AsyncTakeCell::new(&mut sam4l::aes::AES);

// Test state for twiddling the sam4l's AES hardware
static mut TEST: Test<sam4l::aes::Aes<'static>> = Test::new(&AES_CELL);

// Start the test
pub fn start() {
    unsafe {
        TEST.run()
    };
}
