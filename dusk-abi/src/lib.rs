#![no_std]
pub use ethereum_types::U256;

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
