use core::slice;

pub struct CopySlice<'a, T: Copy>{
    ptr: *mut T,
    len: usize,
}

impl<'a, T: Copy> CopySlice {
    pub fn new(buf: &'a [T]) -> CopySlice<'a, T> {
        CopySlice{
            ptr: buf.as_ptr() as *mut T,
            len: buf.len(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_mut(&self) -> &'a mut [T] {
        unsafe {
            slice::from_raw_parts(self.ptr, self.len)
        }
    }
}
