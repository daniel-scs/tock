use kernel::returncode::ReturnCode;
use kernel::hil::crc::{self, CRC};
use sam4l;
use sam4l::crccu::CRCCU;

struct CrcClient;

impl crc::Client for CrcClient {
    fn receive_result(&self, result: u32) {
        if result != 0x1541 {
            blink_sos();
        }
    }
}

static CLIENT: CrcClient = CrcClient;

static DATA: &'static [u8] = b"ABCDEFG";

pub fn crc_test_begin() {
    if CRCCU.get_version() != 0x00000202 {
        blink_sos();
    }

    CRCCU.set_client(&CLIENT);

    let r = CRCCU.compute(&DATA[..]);
    if r != ReturnCode::SUCCESS {
        // let u: usize = From::from(r);
        // panic!("CRC compute failed: {}", u);
        blink_sos();
    }
}

fn blink_sos() { unsafe {
    // blink the panic signal
    let led = &sam4l::gpio::PC[10];
    led.enable_output();
    loop {
        for _ in 0..1000000 {
            led.set();
        }
        for _ in 0..1000000 {
            led.clear();
        }
        for _ in 0..1000000 {
            led.set();
        }
        for _ in 0..1000000 {
            led.clear();
        }
        for _ in 0..1000000 {
            led.set();
        }
        for _ in 0..1000000 {
            led.clear();
        }

        for _ in 0..5000000 {
            led.clear();
        }

        for _ in 0..5000000 {
            led.set();
        }
        for _ in 0..5000000 {
            led.clear();
        }
        for _ in 0..5000000 {
            led.set();
        }
        for _ in 0..5000000 {
            led.clear();
        }
        for _ in 0..5000000 {
            led.set();
        }
        for _ in 0..5000000 {
            led.clear();
        }
    }
} }
