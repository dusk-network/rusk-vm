// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#[doc(hidden)]
pub const BUFFER_SIZE: usize = 1024;

#[doc(hidden)]
pub fn _debug(buf: &[u8]) {
    let len = buf.len() as i32;
    unsafe { crate::hosted::external::debug(&buf[0], len) }
}

/// Macro to format and send debug output to the host
#[macro_export]
macro_rules! debug {
    ($($tt:tt)*) => {
        use core::fmt::Write as _;
        let mut buffer = [0u8; $crate::debug::BUFFER_SIZE];
        let len = {
            let mut bw = $crate::bufwriter::BufWriter::new(&mut buffer);
            write!(bw, $($tt)*).unwrap();
            bw.ofs()
        };
        $crate::debug::_debug(&buffer[0..len])
    };
}
