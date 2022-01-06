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
        }
    }

    fn get(&mut self, ident: &OffsetLen) -> &[u8] {
        let offset = ident.offset();
        let len = ident.len();
        let slice = unsafe { &mut *self.data };
        let to_write = &mut slice[self.written..][..len as usize];
        self.written += len as usize;
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
        TokenBuffer::new(token, unwritten)
    }

    fn commit(&mut self, buffer: &mut TokenBuffer) -> OffsetLen {
        let slice = buffer.written_bytes();
        let len = slice.len();
        assert!(len <= u16::MAX as usize);
        self.written += len;
        abi_put(slice)
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

    fn extend(&self, _buffer: &mut TokenBuffer) -> Result<(), ()> {
        // We can't
        Err(())
    }

    fn return_token(&self, token: Token) {
        let inner = unsafe { &mut *self.inner.get() };
        inner.return_token(token)
    }
}
