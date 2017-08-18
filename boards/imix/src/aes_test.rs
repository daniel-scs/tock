//! Test the AES hardware

use kernel::ReturnCode;
use kernel::hil;
use kernel::hil::symmetric_encryption::{AES128_BLOCK_SIZE, AES128, AES128Ctr};
use sam4l::aes::{AES};

struct Cli { }

static C: Cli = Cli { };

static KEY: [u8; AES128_BLOCK_SIZE] = [1; AES128_BLOCK_SIZE];
static IV: [u8; AES128_BLOCK_SIZE] = [2; AES128_BLOCK_SIZE];
static mut DATA: [u8; AES128_BLOCK_SIZE] = [3; AES128_BLOCK_SIZE];

impl hil::symmetric_encryption::Client for Cli {

    #[allow(unused_unsafe)]
    fn crypt_done(&self) {
        unsafe {
            let data = AES.take_data().unwrap().unwrap();
            debug!("DATA: {:?}", data);

            AES.disable();
        }
    }
}

pub fn run() {
    unsafe {
        AES.enable();

        AES.set_client(&C);
        assert!(AES.set_key(&KEY) == ReturnCode::SUCCESS);
        assert!(AES.set_iv(&IV) == ReturnCode::SUCCESS);
        AES.set_mode_aes128ctr(true);
        AES.start_message();
        assert!(AES.put_data(Some(&mut DATA)) == ReturnCode::SUCCESS);

        let start = 0;
        let stop = AES128_BLOCK_SIZE;
        assert!(AES.crypt(start, stop) == ReturnCode::SUCCESS);

        // await crypt_done()
    }
}
