#![cfg_attr(not(feature = "std"), no_std)]
#![feature(lang_items)]
#![feature(panic_info_message)]

pub use serde::{Deserialize, Serialize};

pub mod encoding;
mod panic;
mod types;

pub use types::{Signature, H256};

// TODO: Extend this error type
pub use fermion::Error;

// declare available host-calls
mod external {
    use super::{Signature, H256};
    extern "C" {
        pub fn set_storage(key: &[u8], value: &[u8]);
        pub fn get_storage(key: &[u8], val: &mut [u8; 1024]) -> bool;
        pub fn caller(buffer: &mut [u8; 32]);
        pub fn balance(buffer: &mut [u8; 32]);
        pub fn debug(text: &str);
        pub fn panic(msg: &[u8]) -> !;
        pub fn call_data(buffer: &mut [u8]);
        pub fn call_contract(target: &H256, amount: &[u8; 16], data: &[u8]);
        pub fn verify_ed25519_signature(
            pub_key: &[u8; 32],
            signature: &[u8; 64],
            buffer: &[u8],
        ) -> bool;
        pub fn ret(data: &[u8]);
    }
}

// implementations
pub fn set_storage<K, V>(key: K, val: V)
where
    K: AsRef<[u8]>,
    V: Serialize,
{
    unsafe {
        let mut val_buf = [0u8; 1024 * 4];

        let val_slice = encoding::encode(&val, &mut val_buf).unwrap();

        external::set_storage(key.as_ref(), val_slice);
    }
}

// implementations
pub fn get_storage<K, V>(key: &K) -> Option<V>
where
    K: AsRef<[u8]> + ?Sized,
    V: for<'de> Deserialize<'de>,
{
    let mut val_buf = [0u8; 1024];
    unsafe {
        if external::get_storage(key.as_ref(), &mut val_buf) {
            Some(encoding::decode(&val_buf).unwrap())
        } else {
            None
        }
    }
}

// implementations
pub fn debug(s: &str) {
    unsafe {
        external::debug(s);
    }
}

pub fn caller() -> H256 {
    let mut buffer = [0u8; 32];
    unsafe { external::caller(&mut buffer) }
    encoding::decode(&buffer[..]).unwrap()
}

pub fn balance() -> H256 {
    let mut buffer = [0u8; 32];
    unsafe { external::balance(&mut buffer) }
    encoding::decode(&buffer[..]).unwrap()
}

pub fn call_data(buffer: &mut [u8]) {
    unsafe { external::call_data(buffer) }
}

pub fn verify_ed25519_signature(
    pub_key: &[u8; 32],
    signature: &Signature,
    buffer: &[u8],
) -> bool {
    unsafe {
        external::verify_ed25519_signature(
            pub_key,
            signature.as_array(),
            buffer,
        )
    }
}

pub fn call_contract(target: &H256, amount: u128, data: &[u8]) {
    let mut buf = [0u8; 16];
    encoding::encode(&amount, &mut buf);
    unsafe { external::call_contract(target, &buf, data) }
}

pub fn ret<T: Serialize>(_ret: T) {
    unimplemented!("ret")
}
