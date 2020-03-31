//! #Dusk ABI
//!
//! ABI functionality for communicating with the host

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(lang_items)]
#![feature(panic_info_message)]

use core::mem;

#[cfg(not(feature = "std"))]
mod bufwriter;
mod contract_call;

#[cfg(not(feature = "std"))]
mod panic;
mod types;

use dataview::Pod;

pub use contract_call::ContractCall;
pub use types::H256;

/// The maximum size of contract call arguments and return values
pub const CALL_DATA_SIZE: usize = 1024 * 16;
/// The maximum size of values in contract storage
pub const STORAGE_VALUE_SIZE: usize = 1024 * 4;
/// The size of keys for contract storage
pub const STORAGE_KEY_SIZE: usize = 32;
/// The maximum length of contract panic messages
pub const PANIC_BUFFER_SIZE: usize = 1024 * 16;
/// The maximum length of contract debug messages
pub const DEBUG_BUFFER_SIZE: usize = 1024 * 16;

// declare available host-calls
mod external {
    use super::*;
    extern "C" {
        pub fn balance(buffer: &mut u8);

        pub fn set_storage(
            key: &[u8; 32],
            value: &[u8; STORAGE_VALUE_SIZE],
            value_len: i32,
        );
        // `get_storage` returns the length of the value
        // 0 is equivalent to no value
        pub fn get_storage(key: &u8, key_len: i32, value: &mut u8) -> i32;
        pub fn caller(buffer: &mut u8);
        pub fn self_hash(buffer: &mut u8);

        #[allow(unused)]
        pub fn panic(msg: &u8, len: i32) -> !;

        pub fn debug(msg: &u8, len: i32);

        pub fn call_data(buffer: &mut u8);
        pub fn call_contract(
            target: &u8,
            amount: u64,
            data: &[u8; CALL_DATA_SIZE],
            data_len: i32,
            ret_pointer: &u8,
        );
        pub fn gas(value: i32);
    }
}

/// Set a contract storage key value
pub fn set_storage<K, V>(key: K, val: V)
where
    K: AsRef<[u8]>,
    V: Pod,
{
    assert!(key.as_ref().len() <= STORAGE_KEY_SIZE);
    let key_slice = key.as_ref();
    let mut key_buf = [0u8; STORAGE_KEY_SIZE];
    key_buf[0..key_slice.len()].copy_from_slice(key.as_ref());
    unsafe {
        let mut val_buf = [0u8; STORAGE_VALUE_SIZE];
        let len = mem::size_of::<V>();
        val_buf[0..len].copy_from_slice(val.as_bytes());

        external::set_storage(&key_buf, &val_buf, len as i32);
    }
}

/// Get a contract storage key value
pub fn get_storage<K, V>(key: &K) -> Option<V>
where
    K: Pod,
    V: Pod,
{
    let mut result = V::zeroed();
    let code: i32 = unsafe {
        external::get_storage(
            &key.as_bytes()[0],
            mem::size_of::<K>() as i32,
            &mut result.as_bytes_mut()[0],
        )
    };
    if code != -1 {
        Some(result)
    } else {
        None
    }
}

/// Returns the caller of the contract
pub fn caller() -> H256 {
    let mut result = H256::zeroed();
    unsafe { external::caller(&mut result.as_bytes_mut()[0]) }
    result
}

/// Returns the hash of the currently executing contract
pub fn self_hash() -> H256 {
    let mut result = H256::zeroed();
    unsafe { external::self_hash(&mut result.as_bytes_mut()[0]) }
    result
}

/// Returns the currently executing contracts balance
pub fn balance() -> u64 {
    let mut result = u64::zeroed();
    unsafe { external::balance(&mut result.as_bytes_mut()[0]) }
    result
}

/// Returns the data the contract was called with
pub fn call_data<D>() -> D
where
    D: Pod,
{
    let mut result = D::zeroed();
    unsafe { external::call_data(&mut result.as_bytes_mut()[0]) }
    result
}

/// Verifies a BLS signature, returns true if successful
pub fn bls_verify(
    pub_key: &[u8; 32],
    signature: &Signature,
    buffer: &[u8],
) -> bool {
    unsafe {
        let len = buffer.len() as i32;
        external::bls_verify(pub_key, signature.as_array_ref(), &buffer[0], len)
    }
}

/// Call another contract at address `target`
pub fn call_contract<R: Pod>(
    target: &H256,
    amount: u64,
    call: &ContractCall<R>,
) -> R {
    let mut result = R::zeroed();
    unsafe {
        external::call_contract(
            &target.as_bytes()[0],
            amount,
            call.data(),
            call.len() as i32,
            &mut result.as_bytes_mut()[0],
        )
    }
    result
}

/// Returns a value and halts execution of the contract
pub fn ret<R: Pod>(ret: R) -> ! {
    unsafe { external::ret(&ret.as_bytes()[0]) }
}

/// Deduct a specified amount of gas from the call
pub fn gas(value: i32) {
    unsafe { external::gas(value) }
}

#[doc(hidden)]
pub fn _debug(buf: &[u8]) {
    let len = buf.len() as i32;
    unsafe { external::debug(&buf[0], len) }
}

/// Macro to format and send debug output to the host
#[macro_export]
macro_rules! debug {
    ($($tt:tt)*) => {
        use core::fmt::Write;
        use $crate::bufwriter::BufWriter;
        let mut buffer = [0u8; $crate::DEBUG_BUFFER_SIZE];
        let len = {
            let mut bw = BufWriter::new(&mut buffer);
            write!(bw, $($tt)*).unwrap();
            bw.ofs()
        };
        $crate::_debug(&buffer[0..len])
    };
}
