use core::ptr;

/// A wrapper around a mutable slice that forces accesses
/// to use volatile reads and writes
pub struct VolatileSlice<'a, T: 'a>(&'a mut [T]);

impl<'a, T> VolatileSlice<'a, T> {
    pub fn new(slice: &'a mut [T]) -> VolatileSlice<'a, T> {
        VolatileSlice(slice)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }
}

pub fn copy_volatile_from_slice<'a, T: Copy>(dst: VolatileSlice<'a, T>, src: &[T]) {
    if src.len() != dst.len() {
        panic!("unequal lengths");
    }
    let p = dst.0.as_mut_ptr();
    for (i, q) in src.iter().enumerate() {
        unsafe {
            ptr::write_volatile(p.offset(i as isize), *q);
        }
    }
}

pub fn copy_from_volatile_slice<'a, T: Copy>(dst: &mut [T], src: VolatileSlice<'a, T>) {
    let p = src.0.as_ptr();
    for (i, q) in dst.iter_mut().enumerate() {
        unsafe {
            *q = ptr::read_volatile(p.offset(i as isize));
        }
    }
}
