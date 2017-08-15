//! Interfaces for encryption and decryption using symmetric ciphers

use returncode::ReturnCode;

pub trait Client {
    fn crypt_done(&self, data: &'static mut [u8]);
}

pub const AES128_BLOCK_SIZE: usize = 16;

pub trait AES128Ctr {
    /// Request an encryption/decryption
    ///
    /// The length `stop_index - start_index` must be a multiple of 16, the
    /// cipher's block size.  If the indices are out of range or out of order,
    /// INVAL will be returned.
    ///
    /// If no buffer is returned, the client's `crypt_done` callback
    /// will eventually be invoked with the same buffer that was passed.
    ///
    /// If SUCCESS is returned, after `crypt_done` is called the portion of the
    /// buffer between `start_index` and `stop_index` will hold the
    /// encryption/decryption of its former contents.
    ///
    /// For correct operation, the `key` and `init_ctr` arguments must not be
    /// modified until callback.
    fn crypt(&self,
             client: &Client,
             encrypting: bool,
             key: &'static [u8],
             init_ctr: &'static [u8],
             data: &'static mut [u8],
             start_index: usize,
             stop_index: usize) -> (ReturnCode, Option<&'static mut [u8]>);
}

pub trait AES128CBC {
    /// Request an encryption/decryption
    ///
    /// The length `stop_index - start_index` must be a multiple of 16, the
    /// cipher's block size.  If the indices are out of range or out of order,
    /// INVAL will be returned.
    ///
    /// If no buffer is returned, the client's `crypt_done` callback
    /// will eventually be invoked with the same buffer that was passed.
    ///
    /// If SUCCESS is returned, after `crypt_done` is called the portion of the
    /// buffer between `start_index` and `stop_index` will hold the
    /// encryption/decryption of its former contents.
    ///
    /// For correct operation, the `key` and `iv` arguments must not be
    /// modified until callback.
    fn crypt(&self,
             client: &Client,
             encrypting: bool,
             key: &'static [u8],
             iv: &'static [u8],
             data: &'static mut [u8],
             start_index: usize,
             stop_index: usize) -> (ReturnCode, Option<&'static mut [u8]>);
}
