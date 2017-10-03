//! Interface for symmetric-cipher encryption
//!
//! see boards/imix/src/aes_test.rs for example usage

use returncode::ReturnCode;

pub trait Client {
    fn crypt_done(&self);
}

pub const AES128_BLOCK_SIZE: usize = 16;

pub trait AES128<'a> {
    // Must be called before any other methods
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
    /// called.  Multiple calls to `put_data()` and `crypt()` may be made
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
    /// be called, and the portion of the data buffer between `start_index`
    /// and `stop_index` will hold the encryption/decryption of its former
    /// contents.
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
    // Call before crypt() to perform AES128Ctr
    fn set_mode_aes128ctr(&self, encrypting: bool);
}

pub trait AES128CBC {
    // Call before crypt() to perform AES128CBC
    fn set_mode_aes128cbc(&self, encrypting: bool);
}
