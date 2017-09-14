//! Test the AES hardware

use core::cell::Cell;
use ReturnCode;
use hil;
use hil::symmetric_encryption::{AES128_BLOCK_SIZE, AES128, AES128Ctr};
use common::{List, ListLink, ListNode};
use common::take_cell::{TakeCell};
use common::async_take_cell::{AsyncTakeCell, AsyncTakeCellClient};

struct Test<'a, Hw: 'a> {
    /// A cell containing a shared mutable reference to the hardware.
    /// When we want to use it, we'll submit a request via `take` and
    /// then wait for the `taken` callback to deliver it to us when it
    /// is our turn
    hw_shared: &'a AsyncTakeCell<'a, Hw>,

    /// When it's our turn to use the reference to the hardware, we'll
    /// stash it here while we're waiting for a callback
    hw_taken: TakeCell<'a, Hw>,

    /// The test state
    mode: Cell<Mode>,

    /// A pointer to be used by AsyncTestCell for keeping track of waiting clients
    next_client_link: ListLink<'a, AsyncTakeCellClient<'a, Hw> + 'a>,
}

#[derive(Copy, Clone)]
enum Mode {
    Encrypting,
    Decrypting,
}

impl<'a, Hw> Test<'a, Hw>
    where Hw: 'a + AES128<'a> + AES128Ctr {

    pub fn new(hw: &'a AsyncTakeCell<'a, Hw>) -> Test<'a, Hw> {
        Test {
            hw_shared: hw,
            hw_taken: TakeCell::empty(),
            mode: Cell::new(Mode::Encrypting),
            next_client_link: ListLink::empty(),
        }
    }

    pub fn run(&self) {
        self.hw_shared.take(self);
        // Wait for taken() to be called when the hardware is available ...
    }
}

/// Provide an `AsyncTakeCellClient` implementation so we can be called
/// when the hardware is ready for us
impl<'a, Hw> AsyncTakeCellClient<'a, Hw> for Test<'a, Hw>
    where Hw: 'a + AES128<'a> + AES128Ctr {

    fn next_client(&'a self) -> &'a ListLink<'a, AsyncTakeCellClient<'a, Hw> + 'a> {
        &self.next_client_link
    }

    fn taken(&'a self, hw: &'a mut Hw) {
        // Stash the reference to the hardware so we can access it in crypt_done()
        self.hw_taken.replace(hw);

        match self.mode.get() {
            Mode::Encrypting => {
                unsafe {
                    hw.enable();

                    hw.set_client(self);
                    assert!(hw.set_key(&KEY) == ReturnCode::SUCCESS);
                    assert!(hw.set_iv(&IV) == ReturnCode::SUCCESS);
                    let encrypting = true;
                    hw.set_mode_aes128ctr(encrypting);
                    hw.start_message();
                    assert!(hw.put_data(Some(&mut DATA)) == ReturnCode::SUCCESS);

                    let start = 0;
                    let stop = DATA.len();
                    assert!(hw.crypt(start, stop) == ReturnCode::SUCCESS);

                    // Await crypt_done() ...
                }
            }
            Mode::Decrypting => {
                unsafe {
                    hw.enable();

                    hw.set_client(self);
                    assert!(hw.set_key(&KEY) == ReturnCode::SUCCESS);
                    assert!(hw.set_iv(&IV) == ReturnCode::SUCCESS);
                    let encrypting = false;
                    hw.set_mode_aes128ctr(encrypting);
                    hw.start_message();
                    assert!(hw.put_data(Some(&mut DATA)) == ReturnCode::SUCCESS);

                    let start = 0;
                    let stop = DATA.len();
                    assert!(hw.crypt(start, stop) == ReturnCode::SUCCESS);

                    // Await crypt_done() ...
                }
            }
        }
    }
}

/// Implement the encryption client interface so we can be called
/// back when the hardware has completed an encryption task.
impl<'a, Hw> hil::symmetric_encryption::Client for Test<'a, Hw>
    where Hw: 'a + AES128<'a> + AES128Ctr {

    #[allow(unused_unsafe)]
    fn crypt_done(&self) {
        if let Some(hw) = self.hw_taken.take() {
            match self.mode.get() {
                Mode::Encrypting => {
                    unsafe {
                        let data = hw.take_data().unwrap().unwrap();
                        if data == CTXT.as_ref() {
                            debug!("Encrypted OK!");
                        } else {
                            debug!("*** BAD CTXT");
                        }
                        hw.disable();

                        // Put back the hardware reference
                        self.hw_shared.replace(hw);
                    }

                    // Continue with decryption test
                    self.mode.set(Mode::Decrypting);
                    self.run();
                }
                Mode::Decrypting => {
                    unsafe {
                        let data = hw.take_data().unwrap().unwrap();
                        if data == PTXT.as_ref() {
                            debug!("Decrypted OK!");
                        } else {
                            debug!("*** BAD PTXT");
                        }
                        hw.disable();

                        // Put back the hardware reference
                        self.hw_shared.replace(hw);
                    }
                }
            }
        } else {
            // This shouldn't happen
            debug!("*** Unexpectedly lost hold of the hardware reference!?");
        }
    }
}

static mut DATA: [u8; 4 * AES128_BLOCK_SIZE] = [
  0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96,
  0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
  0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c,
  0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51,
  0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11,
  0xe5, 0xfb, 0xc1, 0x19, 0x1a, 0x0a, 0x52, 0xef,
  0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17,
  0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10
];

static KEY: [u8; AES128_BLOCK_SIZE] = [
    0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
    0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c
];

static IV: [u8; AES128_BLOCK_SIZE] = [
    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
    0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff
];

static CTXT: [u8; 4 * AES128_BLOCK_SIZE] = [
    0x87, 0x4d, 0x61, 0x91, 0xb6, 0x20, 0xe3, 0x26,
    0x1b, 0xef, 0x68, 0x64, 0x99, 0x0d, 0xb6, 0xce,
    0x98, 0x06, 0xf6, 0x6b, 0x79, 0x70, 0xfd, 0xff,
    0x86, 0x17, 0x18, 0x7b, 0xb9, 0xff, 0xfd, 0xff,
    0x5a, 0xe4, 0xdf, 0x3e, 0xdb, 0xd5, 0xd3, 0x5e,
    0x5b, 0x4f, 0x09, 0x02, 0x0d, 0xb0, 0x3e, 0xab,
    0x1e, 0x03, 0x1d, 0xda, 0x2f, 0xbe, 0x03, 0xd1,
    0x79, 0x21, 0x70, 0xa0, 0xf3, 0x00, 0x9c, 0xee
];

static PTXT: [u8; 4 * AES128_BLOCK_SIZE] = [
    0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96,
    0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
    0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c,
    0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51,
    0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11,
    0xe5, 0xfb, 0xc1, 0x19, 0x1a, 0x0a, 0x52, 0xef,
    0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17,
    0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10
];
