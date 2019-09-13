#![feature(test)]
#![no_std]
extern crate pwasm_std;

// We use black_box to prevent llvm from optimizing away the function calls
use core::hint::black_box;

#[inline(never)]
fn from_both(a: u32) -> u32 {
    black_box(a + 101)
}

#[inline(never)]
fn from_call(b: u32) -> u32 {
    black_box(b + 102) + from_call_deep(b)
}

#[inline(never)]
fn from_call_deep(b: u32) -> u32 {
    black_box(b + 102)
}

#[inline(never)]
fn from_deploy(c: u32) -> u32 {
    black_box(c + 103)
}

#[no_mangle]
pub fn deploy() -> u32 {
    from_both(0) + from_deploy(0)
}

#[no_mangle]
pub fn call() -> u32 {
    from_both(0) + from_call(0)
}
