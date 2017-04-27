//! Diagnostics for the USBC

extern crate kernel;
use kernel::hil;

use sam4l::usbc::{USBC};
use sam4l::usbc::data::*;

use core::slice;
use core::ptr;

static mut EP0_BUF0: [u8; 8] = [99; 8];
static mut EP0_BUF1: [u8; 8] = [77; 8];

#[allow(unused)]
static mut B0: Bank = Bank::new();

#[allow(unused)]
pub fn test_ptr() {
    let p0 = unsafe { &EP0_BUF0 as *const u8 as *mut u8 };
    let p1 = unsafe { &EP0_BUF1 as *const u8 as *mut u8 };
    println!("Buffers at {:?}, {:?}", p0, p1);

    unsafe { B0.set_addr(Buffer(p0)) };

    let p = unsafe { B0.addr.get().0 };

    println!("Buffer at {:?}", p);
    unsafe {
        ptr::write_volatile(p, 0xb0);
        ptr::write_volatile(p.offset(1), 0xb1);
        ptr::write_volatile(p.offset(2), 0xb2);
        ptr::write_volatile(p.offset(3), 0xb3);
        ptr::write_volatile(p.offset(4), 0xb4);
        ptr::write_volatile(p.offset(5), 0xb5);
        ptr::write_volatile(p.offset(6), 0xb6);
        ptr::write_volatile(p.offset(7), 0xb7);
    }

    let slice: &[u8] = unsafe { slice::from_raw_parts(p, 8) };
    println!("Slice: {:?}", slice);
}


struct Dummy { }

impl hil::usb::Client for Dummy {
    fn received_setup(&self /* , descriptor/bank */) {}
    fn received_out(&self /* , descriptor/bank */) {}
}

static DUMMY: Dummy = Dummy {};

// #[allow(unused_unsafe)]
pub fn test() {
    let p0 = unsafe { &EP0_BUF0 as *const u8 as *mut u8 };
    let p1 = unsafe { &EP0_BUF1 as *const u8 as *mut u8 };
    println!("Buffers at {:?}, {:?}", p0, p1);

    unsafe {
        USBC.set_client(&DUMMY);

        USBC.enable(Mode::device_at_speed(Speed::Low));

        USBC.endpoint_bank_set_buffer(EndpointIndex::new(0), BankIndex::Bank0,
                                      p0 // &mut EP0_BUF0
                                      );

        let cfg0 = EndpointConfig::new(BankCount::Single,
                                       EndpointSize::Bytes8,
                                       EndpointDirection::Out,
                                       EndpointType::Control,
                                       EndpointIndex::new(0));
        USBC.endpoint_enable(0, cfg0);

        USBC.attach();

        // USBC.detach();

        // USBC.disable();
    }
}
