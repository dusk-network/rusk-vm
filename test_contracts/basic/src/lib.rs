#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

extern "C" {
    fn host_fn() -> u32;
}

#[no_mangle]
pub fn trampoline() -> u32 {
    unsafe { host_fn() }
}
