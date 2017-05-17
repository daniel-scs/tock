use core;

pub struct CopySlice<'a, T: 'a + Copy>{
    ptr: *mut T,
    len: usize,
    _phantom: core::marker::PhantomData<&'a T>,
}

impl<'a, T: Copy> CopySlice<'a, T> {
    pub fn new(buf: &'a [T]) -> CopySlice<'a, T> {
        CopySlice{
            ptr: buf.as_ptr() as *mut T,
            len: buf.len(),
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_mut(&self) -> &'a mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(self.ptr, self.len)
        }
    }
}