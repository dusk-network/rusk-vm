// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: crate::WeeAlloc = crate::WeeAlloc::INIT;

#[allow(improper_ctypes_definitions)]
#[alloc_error_handler]
#[no_mangle]
pub extern "C" fn oom(_: ::core::alloc::Layout) -> ! {
    ::core::intrinsics::abort();
}

mod panic_handling {
    pub fn signal(msg: &str) {
        let bytes = msg.as_bytes();
        let len = bytes.len() as i32;
        unsafe { sig(&bytes[0], len) }
    }

    extern "C" {
        fn sig(msg: &u8, len: i32);
    }

    use core::fmt::{self, Write};
    use core::panic::PanicInfo;

    impl Write for PanicMsg {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let bytes = s.as_bytes();
            let len = bytes.len();
            self.buf[self.ofs..self.ofs + len].copy_from_slice(bytes);
            self.ofs += len;
            Ok(())
        }
    }

    struct PanicMsg {
        ofs: usize,
        buf: [u8; 1024],
    }

    impl AsRef<str> for PanicMsg {
        fn as_ref(&self) -> &str {
            core::str::from_utf8(&self.buf[0..self.ofs])
                .unwrap_or("PanicMsg.as_ref failed.")
        }
    }

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        let mut msg = PanicMsg {
            ofs: 0,
            buf: [0u8; 1024],
        };

        writeln!(msg, "{}", info).ok();

        signal(msg.as_ref());

        loop {}
    }

    #[lang = "eh_personality"]
    extern "C" fn eh_personality() {}
}
