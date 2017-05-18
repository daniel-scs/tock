use core;
use core::fmt::{Write, Result};

const STORAGE_BYTES: usize = 500;
static STORAGE_ARRAY: [u8; STORAGE_BYTES] = [b'X'; STORAGE_BYTES];
static STORAGE: &'static [u8] = &STORAGE_ARRAY;

#[derive(Copy, Clone)]
pub struct StaticCursor { len: usize }

impl StaticCursor {
    pub fn new() -> Self {
        StaticCursor{ len: 0 }
    }
}

impl StaticCursor {
    pub fn as_str(&self) -> &'static str {
        unsafe {
            static mut BUF: &'static mut [u8; STORAGE_BYTES] = &mut [b'Y'; STORAGE_BYTES];
            let mut sto_p = STORAGE.as_ptr();
            for b in BUF.iter_mut() {
                *b = core::ptr::read_volatile(sto_p);
                sto_p = sto_p.offset(1);
            }

            core::str::from_utf8_unchecked(&BUF[ .. self.len])
        }
    }
}

impl Write for StaticCursor {
    fn write_str(&mut self, s: &str) -> Result {
        unsafe {
            let sb = s.as_bytes();
            if self.len + sb.len() > STORAGE_BYTES {
                panic!("static_fmt: overflow");
            }
            let mut sto_p = STORAGE.as_ptr() as *mut u8;

            sto_p = sto_p.offset(self.len as isize);
            for b in sb.iter() {
                core::ptr::write_volatile(sto_p, *b);
                sto_p = sto_p.offset(1);
            }
            /*
            core::intrinsics::copy_nonoverlapping(sb.as_ptr(),
                                                  sto_p.offset(self.len as isize),
                                                  sb.len());
            */

            self.len += sb.len();
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! static_fmt {
    ($fmt:expr, $($arg:tt)+) => ({
        use core::fmt::Write;

        let mut d = $crate::common::static_fmt::StaticCursor::new();
        write!(d, $fmt, $($arg)+).unwrap();
        d.as_str()
    });
}
