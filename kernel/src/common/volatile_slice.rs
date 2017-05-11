/// A wrapper around a mutable slice that forces accesses
/// to use volatile reads and writes
pub struct VolatileSlice<'a, T>(&'a mut [T]);

impl VolatileSlice {
    pub const fn<'a, T> new(slice: &'a mut [T]) -> VolatileSlice<'a, T> {
        VolatileSlice(slice)
    }
}

pub fn<'a, T: Copy> copy_from_volatile_slice(dst: &mut [T], src: VolatileSlice<'a, T>) {
    let p = src.0.as_ptr();
    for (i, q) in dst.iter_mut().enumerate {
        unsafe {
            *q = ptr::volatile_read(p.offset(i));
        }
    }
}
