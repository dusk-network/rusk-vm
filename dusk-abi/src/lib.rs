//! #Dusk ABI
//!
//! ABI functionality for communicating with the host

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(lang_items)]
#![feature(panic_info_message)]

pub use serde::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
mod bufwriter;
mod contract_call;

/// Module for encoding/decoding values for ABI compability
pub mod encoding;
mod impl_serde;

#[cfg(test)]
mod tests;

#[cfg(not(feature = "std"))]
mod panic;

mod types;

pub use contract_call::{ContractCall, ContractReturn};
pub use types::{Signature, H256};

// TODO: Extend this error type
pub use fermion::Error;

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
        pub fn balance(buffer: &mut [u8; 16]);

        pub fn set_storage(
            key: &[u8; 32],
            value: &[u8; STORAGE_VALUE_SIZE],
            value_len: i32,
        );
        // `get_storage` returns the length of the value
        // 0 is equivalent to no value
        pub fn get_storage(
            key: &[u8; STORAGE_KEY_SIZE],
            value: &mut [u8; STORAGE_VALUE_SIZE],
        ) -> i32;
        pub fn caller(buffer: &mut [u8; 32]);
        pub fn self_hash(buffer: &mut [u8; 32]);

        #[allow(unused)]
        pub fn panic(msg: &u8, len: i32) -> !;

        pub fn debug(msg: &u8, len: i32);

        pub fn call_data(buffer: &mut [u8; CALL_DATA_SIZE]);
        pub fn call_contract(
            target: &[u8; 32],
            amount: &[u8; 16],
            data: &[u8; CALL_DATA_SIZE],
            data_len: i32,
        );
        pub fn verify_ed25519_signature(
            pub_key: &[u8; 32],
            signature: &[u8; 64],
            buffer: &u8,
            buffer_len: i32,
        ) -> bool;
        pub fn ret(data: &[u8; CALL_DATA_SIZE]) -> !;

        pub fn gas(value: i32);

    }
}

/// Set a contract storage key value
pub fn set_storage<K, V>(key: K, val: V)
where
    K: AsRef<[u8]>,
    V: Serialize,
{
    assert!(key.as_ref().len() <= STORAGE_KEY_SIZE);
    let key_slice = key.as_ref();
    let mut key_buf = [0u8; STORAGE_KEY_SIZE];
    key_buf[0..key_slice.len()].copy_from_slice(key.as_ref());
    unsafe {
        let mut val_buf = [0u8; STORAGE_VALUE_SIZE];

        let len = encoding::encode(&val, &mut val_buf).unwrap().len();

        external::set_storage(&key_buf, &val_buf, len as i32);
    }
}

/// Get a contract storage key value
pub fn get_storage<K, V>(key: &K) -> Option<V>
where
    K: AsRef<[u8]> + ?Sized,
    V: for<'de> Deserialize<'de>,
{
    let slice = key.as_ref();
    let len = slice.len();
    assert!(len <= STORAGE_KEY_SIZE);
    let mut key_buf = [0u8; STORAGE_KEY_SIZE];
    key_buf[0..len].copy_from_slice(key.as_ref());

    let mut val_buf = [0u8; STORAGE_VALUE_SIZE];
    unsafe {
        let len = external::get_storage(&key_buf, &mut val_buf) as usize;
        if len > 0 {
            Some(encoding::decode(&val_buf[0..len]).unwrap())
        } else {
            None
        }
    }
}

/// Returns the caller of the contract
pub fn caller() -> H256 {
    let mut buffer = [0u8; 32];
    unsafe { external::caller(&mut buffer) }
    encoding::decode(&buffer[..]).unwrap()
}

/// Returns the hash of the currently executing contract
pub fn self_hash() -> H256 {
    let mut buffer = [0u8; 32];
    unsafe { external::self_hash(&mut buffer) }
    encoding::decode(&buffer[..]).unwrap()
}

/// Returns the currently executing contracts balance
pub fn balance() -> u128 {
    let mut buffer = [0u8; 16];
    unsafe { external::balance(&mut buffer) }
    encoding::decode(&buffer[..]).unwrap()
}

/// Returns the data the contract was called with
pub fn call_data<'de, D>(buffer: &'de mut [u8; CALL_DATA_SIZE]) -> D
where
    D: Deserialize<'de>,
{
    unsafe { external::call_data(buffer) }
    encoding::decode(buffer).unwrap()
}

/// Verifies an ed25519_signature, returns true if successful
pub fn verify_ed25519_signature(
    pub_key: &[u8; 32],
    signature: &Signature,
    buffer: &[u8],
) -> bool {
    unsafe {
        let len = buffer.len() as i32;
        external::verify_ed25519_signature(
            pub_key,
            signature.as_array_ref(),
            &buffer[0],
            len,
        )
    }
}

/// Call another contract at address `target`
pub fn call_contract<'de, R: 'de + Deserialize<'de>>(
    target: &H256,
    amount: u128,
    call: &'de ContractCall<R>,
) -> R {
    let mut target_buf = [0u8; 32];
    encoding::encode(&target, &mut target_buf).unwrap();
    let mut amount_buf = [0u8; 16];
    encoding::encode(&amount, &mut amount_buf).unwrap();
    unsafe {
        external::call_contract(
            &target_buf,
            &amount_buf,
            call.data(),
            call.len() as i32,
        )
    }
    encoding::decode(call.data()).unwrap()
}

/// Returns a value and halts execution of the contract
pub fn ret<T: Serialize>(ret: T) -> ! {
    let mut ret_buffer = [0u8; CALL_DATA_SIZE];
    encoding::encode(&ret, &mut ret_buffer).unwrap();
    unsafe { external::ret(&ret_buffer) }
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
