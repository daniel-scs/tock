//! Implementation of the AESA peripheral on the SAM4L.

use core::cell::Cell;
use core::mem;

use kernel::common::VolatileCell;
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use kernel::hil::symmetric_encryption::{Client, Request, BLOCK_SIZE, ConfidentialityMode};
use nvic;
use pm;
use scif;

/// The registers used to interface with the hardware
#[repr(C, packed)]
struct AesRegisters {
    ctrl: VolatileCell<u32>, //       0x00
    mode: VolatileCell<u32>, //       0x04
    databufptr: VolatileCell<u32>, // 0x08
    sr: VolatileCell<u32>, //         0x0C
    ier: VolatileCell<u32>, //        0x10
    idr: VolatileCell<u32>, //        0x14
    imr: VolatileCell<u32>, //        0x18
    _reserved0: VolatileCell<u32>, // 0x1C
    key0: VolatileCell<u32>, //       0x20
    key1: VolatileCell<u32>, //       0x24
    key2: VolatileCell<u32>, //       0x28
    key3: VolatileCell<u32>, //       0x2c
    key4: VolatileCell<u32>, //       0x30
    key5: VolatileCell<u32>, //       0x34
    key6: VolatileCell<u32>, //       0x38
    key7: VolatileCell<u32>, //       0x3c
    initvect0: VolatileCell<u32>, //  0x40
    initvect1: VolatileCell<u32>, //  0x44
    initvect2: VolatileCell<u32>, //  0x48
    initvect3: VolatileCell<u32>, //  0x4c
    idata: VolatileCell<u32>, //      0x50
    _reserved1: [u32; 3], //          0x54 - 0x5c
    odata: VolatileCell<u32>, //      0x60
    _reserved2: [u32; 3], //          0x64 - 0x6c
    drngseed: VolatileCell<u32>, //   0x70
}

// Section 7.1 of datasheet
const AES_BASE: u32 = 0x400B0000;

const IBUFRDY: u32 = 1 << 16;
const ODATARDY: u32 = 1 << 0;

pub struct Aes<'a> {
    registers: *mut AesRegisters,

    // The request in progress, if any.
    // (This may be extended in future to hold multiple outstanding requests.)
    request: TakeCell<'a, Request<'a>>,

    // An index into the request buffer marking how much data
    // has been written to the AESA
    write_index: Cell<usize>,

    // An index into the request buffer marking how much data
    // has been read back from the AESA
    read_index: Cell<usize>,
}

pub static mut AES: Aes = Aes::new();

impl<'a> Aes<'a> {
    pub const fn new() -> Aes<'a> {
        Aes {
            registers: AES_BASE as *mut AesRegisters,
            request: TakeCell::empty(),
            write_index: Cell::new(0),
            read_index: Cell::new(0),
        }
    }

    fn enable_clock(&self) {
        unsafe {
            pm::enable_clock(pm::Clock::HSB(pm::HSBClock::AESA));
            scif::generic_clock_enable_divided(scif::GenericClock::GCLK4,
                                               scif::ClockSource::CLK_CPU,
                                               1);
            scif::generic_clock_enable(scif::GenericClock::GCLK4, scif::ClockSource::CLK_CPU);
        }
    }

    fn disable_clock(&self) {
        unsafe {
            scif::generic_clock_disable(scif::GenericClock::GCLK4);
            pm::disable_clock(pm::Clock::HSB(pm::HSBClock::AESA));
        }
    }

    pub fn enable(&self) {
        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };

        self.enable_clock();
        unsafe {
            nvic::enable(nvic::NvicIdx::AESA);
        }
        regs.ctrl.set(0x01);
    }

    pub fn disable(&self) {
        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };

        regs.ctrl.set(0x00);
        unsafe {
            nvic::disable(nvic::NvicIdx::AESA);
        }
        self.disable_clock();
    }

    fn enable_interrupts(&self) {
        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };

        unsafe {
            nvic::clear_pending(nvic::NvicIdx::AESA);
        }

        // We want both interrupts.
        regs.ier.set(IBUFRDY | ODATARDY);
    }

    fn disable_interrupts(&self) {
        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };

        // Disable both interrupts
        regs.idr.set(IBUFRDY | ODATARDY);
    }

    fn disable_input_interrupt(&self) {
        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };

        // Tell the AESA not to send an interrupt looking for more input
        regs.idr.set(IBUFRDY);
    }

    fn set_mode(&self, encrypting: bool, mode: ConfidentialityMode) {
        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };

        let encrypt = if encrypting { 1 } else { 0 };
        let dma = 0;
        let cmeasure = 0xF;
        regs.mode.set(encrypt << 0 | dma << 3 | (mode as u32) << 4 | cmeasure << 16);
    }

    fn set_iv(&self, iv: &[u8; BLOCK_SIZE]) {
        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };

        // Set the initial value from the array.
        for i in 0..4 {
            let mut c = iv[i * 4 + 0] as usize;
            c |= (iv[i * 4 + 1] as usize) << 8;
            c |= (iv[i * 4 + 2] as usize) << 16;
            c |= (iv[i * 4 + 3] as usize) << 24;
            match i {
                0 => regs.initvect0.set(c as u32),
                1 => regs.initvect1.set(c as u32),
                2 => regs.initvect2.set(c as u32),
                3 => regs.initvect3.set(c as u32),
                _ => {}
            }
        }
    }

    fn set_key(&self, key: &[u8; BLOCK_SIZE]) {
        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };

        for i in 0..4 {
            let mut k = key[i * 4 + 0] as usize;
            k |= (key[i * 4 + 1] as usize) << 8;
            k |= (key[i * 4 + 2] as usize) << 16;
            k |= (key[i * 4 + 3] as usize) << 24;
            match i {
                0 => regs.key0.set(k as u32),
                1 => regs.key1.set(k as u32),
                2 => regs.key2.set(k as u32),
                3 => regs.key3.set(k as u32),
                _ => {}
            }
        }
    }

    // Alert the AESA that we are beginning a new message
    fn notify_new_message(&self) {
        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };

        // Notify of a new message.
        regs.ctrl.set((1 << 2) | (1 << 0));
    }

    fn input_buffer_ready(&self) -> bool {
        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };
        let status = regs.sr.get();

        status & (1 << 16) != 0
    }

    fn output_data_ready(&self) -> bool {
        let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };
        let status = regs.sr.get();

        status & (1 << 0) != 0
    }

    // Copy a block from the request buffer to the AESA input register,
    // if there is a block left in the buffer.  Either way, this function
    // returns true if more blocks remain to send
    fn write_block(&self) -> bool {
        self.request.map_or_else(|| {
            debug!("Called write_block() with no request");
            false
        },
        |request| {

            let index = self.write_index.get();
            let more = index + BLOCK_SIZE <= request.stop_index;
            if !more {
                return false;
            }

            let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };
            let data = request.data;
            for i in 0..4 {
                let mut v = data[index + (i * 4) + 0] as usize;
                v |= (data[index + (i * 4) + 1] as usize) << 8;
                v |= (data[index + (i * 4) + 2] as usize) << 16;
                v |= (data[index + (i * 4) + 3] as usize) << 24;
                regs.idata.set(v as u32);
            }

            self.write_index.set(index + BLOCK_SIZE);

            let more = self.write_index.get() + BLOCK_SIZE <= request.stop_index;
            more
        })
    }

    // Copy a block from the AESA output register back into the request buffer
    // if there is any room left.  Return true if we are still waiting for more
    // blocks after this
    fn read_block(&self) -> bool
    {
        self.request.map_or_else(|| {
            debug!("Called read_block() with no request");
            false
        },
        |request| {
            let index = self.read_index.get();
            let more = index + BLOCK_SIZE <= request.stop_index;
            if !more {
                return false;
            }

            let regs: &mut AesRegisters = unsafe { mem::transmute(self.registers) };
            let data = request.data;
            for i in 0..4 {
                let v = regs.odata.get();
                data[index + (i * 4) + 0] = (v >> 0) as u8;
                data[index + (i * 4) + 1] = (v >> 8) as u8;
                data[index + (i * 4) + 2] = (v >> 16) as u8;
                data[index + (i * 4) + 3] = (v >> 24) as u8;
            }

            self.read_index.set(index + BLOCK_SIZE);

            let more = self.read_index.get() + BLOCK_SIZE <= request.stop_index;
            more
        })
    }

    // Enqueue an encryption request.  Returns true if request is accepted and
    // the client will be alerted upon completion.
    fn enqueue_request(&self, request: &mut Request<'a>) -> bool {
        if self.request.is_some() {
            // In future, append this request to a linked list
            false
        } else {
            // "Enqueue" the request
            self.request.put(Some(request));

            // The queue is now non-empty, so begin processing
            self.process_waiting_requests();

            // We accepted the request
            true
        }
    }
 
    // Dequeue a completed request and begin processing another
    fn finish_request(&self) {
        // Remove the completed request
        if let Some(request) = self.request.take() {

            // Alert the client of the completion and return the buffer
            request.client.crypt_done(request.data);
        }

        self.process_waiting_requests();
    }

    // Begin processing the request at the head of the "queue"
    fn process_waiting_requests(&self) {
        self.request.map_or_else(|| {
            self.disable_interrupts();
            self.disable();
        },
        |request| {
            self.enable();
            self.set_mode(request.encrypting, request.mode);
            self.set_key(request.key);
            self.set_iv(request.iv);
            self.notify_new_message();

            self.write_index.set(request.start_index);
            self.read_index.set(request.start_index);

            self.enable_interrupts();
        });
    }

    // Handle an interrupt, which will indicate either that the AESA's input
    // buffer is ready for more data, or that it has completed a block of output
    // for us to consume
    pub fn handle_interrupt(&self) {
        self.request.map_or_else(|| {
            debug!("Received interrupt with no request pending");
            self.disable_interrupts();
        },
        |request| {
            if self.input_buffer_ready() {
                // The AESA says it is ready to receive another block

                if !self.write_block() {
                    // We've now written the entirety of the request buffer
                    self.disable_input_interrupt();
                }
            }

            if self.output_data_ready() {
                // The AESA says it has a completed block to give us

                if !self.read_block() {
                    // We've read back all the blocks
                    self.disable_interrupts();

                    self.finish_request();
                }
            }
        });
    }
}

impl<'a> hil::symmetric_encryption::AES128Ctr<'a> for Aes<'a> {
    fn create_request(client: &'a hil::symmetric_encryption::Client<'a>,
                      encrypting: bool,
                      key: &'a [u8; BLOCK_SIZE],
                      init_ctr: &'a [u8; BLOCK_SIZE],
                      data: &'a mut [u8],
                      start_index: usize,
                      stop_index: usize) -> Option<Request<'a>> {
        Request::new(client,
                     ConfidentialityMode::Ctr,
                     encrypting,
                     key,
                     init_ctr,
                     data,
                     start_index,
                     stop_index)
    }

    fn crypt(&self, request: &mut Request<'a>) -> bool {
        self.enqueue_request(request)
    }
}

impl<'a> hil::symmetric_encryption::AES128CBC<'a> for Aes<'a> {
    fn create_request(client: &'a hil::symmetric_encryption::Client<'a>,
                      encrypting: bool,
                      key: &'a [u8; BLOCK_SIZE],
                      iv: &'a [u8; BLOCK_SIZE],
                      data: &'a mut [u8],
                      start_index: usize,
                      stop_index: usize) -> Option<Request<'a>> {
        Request::new(client,
                     ConfidentialityMode::CBC,
                     encrypting,
                     key,
                     iv,
                     data,
                     start_index,
                     stop_index)
    }

    fn crypt(&self, request: &mut Request<'a>) -> bool {
        self.enqueue_request(request)
    }
}

interrupt_handler!(aes_handler, AESA);
