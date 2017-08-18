//! Interface for symmetric-cipher encryption
//!
//! Example usage:
//!
//! e.enable();
//! e.set_client(c);
//! assert!(e.set_key(key) == ReturnCode::SUCCESS);
//! assert!(e.set_iv(iv) == ReturnCode::SUCCESS);
//! e.set_mode_aes128ctr(true);
//! e.start_message();
//! _ = e.replace_data(Some(data1)).unwrap();
//! e.crypt(start1, stop1);
//!     // await crypt_done()
//! data1 = e.replace_data(Some(data2)).unwrap().unwrap()
//! e.crypt(start2, stop2);
//!     // await crypt_done()
//! data2 = e.replace_data(None).unwrap().unwrap()
//! e.disable();

use returncode::ReturnCode;

pub trait Client {
    fn crypt_done(&self);
}

pub const AES128_BLOCK_SIZE: usize = 16;

pub trait AES128<'a> {
    fn enable(&self);
    fn disable(&self);

    fn set_client(&'a self, client: &'a Client);
    fn set_key(&'a self, key: &'a [u8]) -> ReturnCode;
    fn set_iv(&'a self, iv: &'a [u8]) -> ReturnCode;

    /// Set the optional data buffer.
    /// The option should be full whenever `crypt()` is called.
    /// Returns SUCCESS if the buffer was installed, or EBUSY
    /// if the encryption unit is still busy.
    fn put_data(&'a self, data: Option<&'a mut [u8]>) -> ReturnCode;

    /// Return the data buffer, if any.
    /// Returns EBUSY if the encryption unit is still busy.
    fn take_data(&'a self) -> Result<Option<&'a mut [u8]>, ReturnCode>;

    /// Begin a new message (with the configured IV) when `crypt()` is next
    /// called.  Multiple calls to `set_data()` and `crypt()` may be made
    /// between calls to `start_message()`, allowing the encryption context
    /// to extend over non-contiguous extents of data.
    fn start_message(&self);

    /// Request an encryption/decryption
    ///
    /// The indices `start_index` and `stop_index` must be valid offsets in
    /// a buffer previously passed in with `set_data`, and the length
    /// `stop_index - start_index` must be a multiple of
    /// `AES128_BLOCK_SIZE`.  Otherwise, INVAL will be returned.
    ///
    /// If SUCCESS is returned, the client's `crypt_done` method will eventually
    /// be called with the same buffer previously passed in with `set_data`,
    /// and the portion of the buffer between `start_index` and `stop_index`
    /// will hold the encryption/decryption of its former contents.
    ///
    /// For correct operation, the methods `set_key` and `set_iv` must have
    /// previously been called to set the buffers containing the
    /// key and the IV (or initial counter value), and these buffers must
    /// not be modified until `crypt_done` is called.
    fn crypt(&self,
             start_index: usize,
             stop_index: usize) -> ReturnCode;
}

pub trait AES128Ctr {
    fn set_mode_aes128ctr(&self, encrypting: bool);
}

pub trait AES128CBC {
    fn set_mode_aes128cbc(&self, encrypting: bool);
}
