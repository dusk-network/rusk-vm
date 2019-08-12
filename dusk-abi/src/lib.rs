#![no_std]
use core::slice;

extern "C" {
    fn abi_debug(len: u32);
    fn abi_memory() -> *mut u8;
}

fn get_memory() -> &'static mut [u8] {
    unsafe {
        let ofs = abi_memory();
        let len = (*ofs) as usize;
        slice::from_raw_parts_mut(ofs, len)
    }
}

pub fn debug(string: &str) {
    unsafe {
        let memory = get_memory();
        memory[0] = 65;
        abi_debug(1);
    }
}
