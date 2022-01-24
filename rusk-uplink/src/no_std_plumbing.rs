use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::debug!("{:?}", info)
    loop {}
}

extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[alloc_error_handler]
fn foo(_: core::alloc::Layout) -> ! {
    loop {}
}
