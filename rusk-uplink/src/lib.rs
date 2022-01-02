mod ext {
    #[link(wasm_import_module = "env")]
    extern "C" {
        pub fn debug(ofs: &u8, len: i32);
    }
}

pub fn debug(string: &'static str) {
    let bytes = string.as_bytes();
    unsafe { ext::debug(&bytes[0], bytes.len() as i32) }
}

/// Store backend over FFI
#[cfg(not(feature = "host"))]
mod ffi_store;
mod ffi_wasmer_store;

/// Store backend over FFI
#[cfg(not(feature = "host"))]
pub use ffi_store::*;
pub use ffi_wasmer_store::*;

pub mod helpers;
pub use helpers::*;

pub mod definitions;
pub use definitions::*;

pub mod hosted;
pub use hosted::*;

pub mod bufwriter;
pub use bufwriter::*;

pub mod debug;
pub use debug::*;
