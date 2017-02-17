use kernel::returncode::ReturnCode;
use kernel::hil::crc::{self, CRC};
use sam4l::crccu::CRCCU;

struct CrcClient;

impl crc::Client for CrcClient {
    fn receive_result(&self, result: u32) {
        assert_eq!(result, 0x1541);
    }
}

static CLIENT: CrcClient = CrcClient;

static DATA: &'static [u8] = b"ABCDEFG";

pub fn crc_test_begin() {
    assert_eq!(CRCCU.get_version(), 0x00000202);

    CRCCU.set_client(&CLIENT);

    let r = CRCCU.compute(&DATA[..]);
    if r != ReturnCode::SUCCESS {
        let u: usize = From::from(r);
        panic!("CRC compute failed: {}", u);
    }
}
