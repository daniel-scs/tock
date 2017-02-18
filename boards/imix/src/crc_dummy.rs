use kernel::returncode::ReturnCode;
use kernel::hil::crc::{self, CRC};
use sam4l;
use sam4l::crccu::CRCCU;

struct CrcClient;

impl crc::Client for CrcClient {
    fn receive_result(&self, result: u32) {
        if result != 0x1541 {
            blink_loop_n(7);
        }
        blink_loop_n(5);
    }

    fn interrupt(&self) {
        blink_loop_n(6);
    }
}

static CLIENT: CrcClient = CrcClient;

static DATA: &'static [u8] = b"ABCDEFG";

pub fn crc_test_begin() {
    CRCCU.enable_unit();
    blink_n(4, 1);

    if CRCCU.get_version() != 0x00000202 {
        blink_loop_n(2);
    }

    CRCCU.set_client(&CLIENT);

    if CRCCU.compute(&DATA[..]) != ReturnCode::SUCCESS {
        blink_loop_n(3);
    }
}

fn blink_loop_n(n: u8) {
    blink_n(n, 0);
}

fn blink_n(n: u8, times: u32) {
    unsafe {
        // blink the panic signal
        let led = &sam4l::gpio::PC[10];
        led.enable_output();

        let mut i = times;
        while times == 0 || i > 0 {
            if times > 0 {
                i -= 1;
            }

            for _ in 0..n {
                for _ in 0..1000000 {
                    led.set();
                }
                for _ in 0..1000000 {
                    led.clear();
                }
            }

            for _ in 0..2000000 {
                led.clear();
            }
        }
    }
}
