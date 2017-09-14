//! Test the AES hardware

use kernel::common::async_take_cell::{AsyncTakeCell};
use kernel::aes_test::{Test};
use sam4l;

// Start the test
pub unsafe fn start() {

    // A cell holding a reference to the sam4l's AES hardware
    let aes_shared = static_init!(AsyncTakeCell<'static, sam4l::aes::Aes>,
                                  AsyncTakeCell::new(&mut sam4l::aes::AES));

    // Test state for twiddling the sam4l's AES hardware
    let test = static_init!(Test<sam4l::aes::Aes<'static>>,
                            Test::new(aes_shared));

    test.run()
}
