// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::cell::UnsafeCell;

use microkelvin::{OffsetLen, Store, Token, TokenBuffer};
use rkyv::Fallible;

extern "C" {
    fn _put(slice: &u8, len: u16) -> u64;
    fn _get(offset: u64, len: u16, buf: &mut u8);
}

fn abi_put(slice: &[u8]) -> OffsetLen {
    assert!(slice.len() <= u16::MAX as usize);
    let len = slice.len() as u16;
    let ofs = unsafe { _put(&slice[0], len) };

    OffsetLen::new(ofs, len)
}

fn abi_get(offset: u64, buf: &mut [u8]) {
    let len = buf.len() as u16;
    unsafe { _get(offset, len, &mut buf[0]) }
}

struct AbiStoreInner {
    data: *mut [u8],
    written: usize,
    token: Token,
    data_vec: *mut Vec<u8>,
    data_ofs: usize,
}

pub struct AbiStore {
    inner: UnsafeCell<AbiStoreInner>,
}

impl Fallible for AbiStore {
    type Error = core::convert::Infallible;
}

const MIN_RESIZE: usize = 128;

impl AbiStoreInner {
    fn new(buf: &mut [u8], buf_vec: &mut Vec<u8>, data_ofs: usize) -> Self {
        AbiStoreInner {
            data: buf,
            written: 0,
            token: Token::new(),
            data_vec: buf_vec,
            data_ofs,
        }
    }

    fn resize_by(&mut self, by: usize) {
        unsafe {
            (*self.data_vec).resize((*self.data_vec).len() + by, 0u8);
            self.data = &mut (*self.data_vec).as_mut_slice()[self.data_ofs..];
        }
    }

    fn get(&mut self, ident: &OffsetLen) -> &[u8] {
        let offset = ident.offset();
        let len = ident.len() as usize;
        let current_len = unsafe { &mut *self.data }.len();

        if (self.written + len) > current_len {
            self.resize_by(core::cmp::max(
                self.written + len - current_len,
                MIN_RESIZE,
            ));
        }
        let slice = unsafe { &mut *self.data };
        let to_write = &mut slice[self.written..][..len as usize];

        self.written += len;

        abi_get(offset, to_write);

        to_write
    }

    fn return_token(&mut self, token: Token) {
        self.token.return_token(token)
    }

    fn request_buffer(&mut self) -> TokenBuffer {
        let slice = unsafe { &mut *self.data };
        let unwritten = &mut slice[self.written..];
        let token = self.token.take().expect("token error");
        assert_eq!(self.written, 0, "Buffer must be requested when written is zero, if not, TokenBuffer will have to keep this offset to make extend work");
        TokenBuffer::new(token, unwritten)
    }

    fn commit(&mut self, buffer: &mut TokenBuffer) -> OffsetLen {
        let slice = buffer.written_bytes();
        let len = slice.len() as usize;
        let abi_put_ofslen = abi_put(slice);
        assert!(len <= u16::MAX as usize);
        self.written += len;
        abi_put_ofslen
    }
}

impl AbiStore {
    pub fn new(buf: &mut [u8], buf_vec: &mut Vec<u8>, data_ofs: usize) -> Self {
        AbiStore {
            inner: UnsafeCell::new(AbiStoreInner::new(buf, buf_vec, data_ofs)),
        }
    }
}

impl Store for AbiStore {
    type Identifier = OffsetLen;

    fn get(&self, ident: &OffsetLen) -> &[u8] {
        let inner = unsafe { &mut *self.inner.get() };
        inner.get(ident)
    }

    fn request_buffer(&self) -> TokenBuffer {
        let inner = unsafe { &mut *self.inner.get() };
        inner.request_buffer()
    }

    fn persist(&self) -> Result<(), ()> {
        Err(())
    }

    fn commit(&self, buffer: &mut TokenBuffer) -> Self::Identifier {
        let inner = unsafe { &mut *self.inner.get() };
        inner.commit(buffer)
    }

    fn extend(
        &self,
        buffer: &mut TokenBuffer,
        size_needed: usize,
        _size_present: usize,
    ) -> Result<(), ()> {
        let inner = unsafe { &mut *self.inner.get() };
        let slice = unsafe { &mut *inner.data };
        inner.resize_by(size_needed + slice.len() + MIN_RESIZE);
        let slice = unsafe { &mut *inner.data };
        buffer.reset_buffer(slice); // only if written at TokenBuffer creation time was zero, otherwise need
                                    // to add offset to TokenBuffer
        Ok(())
    }

    fn return_token(&self, token: Token) {
        let inner = unsafe { &mut *self.inner.get() };
        inner.return_token(token)
    }
}
