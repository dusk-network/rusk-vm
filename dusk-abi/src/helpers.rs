use core::mem;

use dataview::Pod;

/// Helper methods for Pod
pub trait PodExt: Pod + Sized {
    /// Returns a byte pointer to the value
    fn as_byte_ptr(&self) -> &u8 {
        &self.as_bytes()[0]
    }

    /// Returns a mutable byte pointer to the value
    fn as_byte_ptr_mut(&mut self) -> &mut u8 {
        &mut self.as_bytes_mut()[0]
    }

    /// Construct a value from a byte slice
    fn from_slice(slice: &[u8]) -> Self {
        let mut s = Self::zeroed();
        s.as_bytes_mut()
            .copy_from_slice(&slice[0..mem::size_of::<Self>()]);
        s
    }
}

impl<T> PodExt for T where T: Pod {}
