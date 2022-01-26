// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::fmt;

/// A small struct that can `Write` to a buffer
pub struct BufWriter<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> BufWriter<'a> {
    /// Creates a new `BufWriter`
    pub fn new(buf: &'a mut [u8]) -> Self {
        BufWriter { buf, offset: 0 }
    }

    /// Returns the offset of the `BufWriter`
    pub fn ofs(&self) -> usize {
        self.offset
    }
}

impl<'a> fmt::Write for BufWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();

        // Skip over already-copied data
        let remainder = &mut self.buf[self.offset..];
        // Check if there is space remaining (return error instead of panicking)
        if remainder.len() < bytes.len() {
            return Err(core::fmt::Error);
        }
        // Make the two slices the same length
        let remainder = &mut remainder[..bytes.len()];
        // Copy
        remainder.copy_from_slice(bytes);

        // Update offset to avoid overwriting
        self.offset += bytes.len();

        Ok(())
    }
}
