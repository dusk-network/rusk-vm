#![no_std]
use core::slice;

#[repr(C)]
pub struct U256([u8; 32]);

impl U256 {
    pub fn zero() -> Self {
        U256([0u8; 32])
    }

    pub fn max() -> Self {
        U256([255u8; 32])
    }
}

mod external {
    use super::U256;
    extern "C" {
        pub fn abi_set_storage(key: *const U256, key: *const U256);
    }
}

pub fn set_storage(key: &U256, val: &U256) {
    unsafe {
        external::abi_set_storage(key, val);
    }
}
