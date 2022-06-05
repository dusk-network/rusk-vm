// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#[doc(hidden)]
pub const BUFFER_SIZE: usize = 1024;

#[cfg(target_family = "wasm")]
/// Macro to format and send debug output to the host
#[macro_export]
macro_rules! debug {
    ($($tt:tt)*) => {
	#[allow(unused)]
        use core::fmt::Write as _;
        let mut buffer = [0u8; $crate::debug::BUFFER_SIZE];
        let len = {
            let mut bw = $crate::bufwriter::BufWriter::new(&mut buffer);
            write!(bw, $($tt)*).unwrap();
            bw.ofs()
        };
	unsafe {
            $crate::hosted::external::debug(&buffer[0], len as i32)
	}
    };
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! debug {
    ($($tt:tt)*) => {
        ()
    };
}
