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

const PAGE_SIZE: usize = 1024 * 64;
#[derive(Debug)]
struct Page {
    bytes: Box<[u8; PAGE_SIZE]>,
    written: usize,
}
impl Page {
    fn new() -> Self {
        Page {
            bytes: Box::new([0u8; PAGE_SIZE]),
            written: 0,
        }
    }
    fn unwritten_tail(&mut self) -> &mut [u8] {
        &mut self.bytes[self.written..]
    }
}
struct AbiStoreInner {
    data: *mut [u8],
    written: usize,
    token: Token,
    pages: Vec<Page>,
}

pub struct AbiStore {
    inner: UnsafeCell<AbiStoreInner>,
}

impl Fallible for AbiStore {
    type Error = core::convert::Infallible;
}

impl AbiStoreInner {
    fn new(buf: &mut [u8]) -> Self {
        AbiStoreInner {
            data: buf,
            written: 0,
            token: Token::new(),
            pages: Vec::new(),
        }
    }

    fn unwritten_tail<'a>(&'a mut self) -> &'a mut [u8] {
        let bytes = match self.pages.last_mut() {
            Some(page) => page.unwritten_tail(),
            None => {
                self.pages = vec![Page::new()];
                self.pages[0].unwritten_tail()
            }
        };
        let extended: &'a mut [u8] = unsafe { core::mem::transmute(bytes) };
        extended
    }
    fn extend(&mut self) {
        self.pages.push(Page::new());
        self.data = self.unwritten_tail();
        self.written = 0;
    }
    fn get(&mut self, ident: &OffsetLen) -> &[u8] {
        let offset = ident.offset();
        let len = ident.len() as usize;
        let current_len = unsafe { &mut *self.data }.len();

        if (self.written + len) > current_len {
            self.extend();
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
        let buf = buffer.as_mut() as *mut _ as *mut [u8];
        buffer.remap(unsafe { &mut *buf }); // buffer.rewind();
        assert!(len <= u32::MAX as usize);
        self.written -= core::cmp::min(len, self.written);
        abi_put_ofslen
    }
}

impl AbiStore {
    pub fn new(buf: &mut [u8]) -> Self {
        AbiStore {
            inner: UnsafeCell::new(AbiStoreInner::new(buf)),
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

    fn extend(&self, buffer: &mut TokenBuffer) -> Result<(), ()> {
        let inner = unsafe { &mut *self.inner.get() };
        inner.extend();
        let slice = unsafe { &mut *inner.data };
        buffer.remap(slice);
        Ok(())
    }

    fn return_token(&self, token: Token) {
        let inner = unsafe { &mut *self.inner.get() };
        inner.return_token(token)
    }
}
