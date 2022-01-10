use core::cell::UnsafeCell;

use microkelvin::{OffsetLen, Store, Token, TokenBuffer};
use rkyv::Fallible;

extern "C" {
    fn s_put(slice: &u8, len: u16) -> u64;
    fn s_get(offset: u64, buf: &mut u8);
}

fn abi_put(slice: &[u8]) -> OffsetLen {
    assert!(slice.len() <= u16::MAX as usize);
    let len = slice.len() as u16;
    let ofs = unsafe { s_put(&slice[0], len) };
    OffsetLen::new(ofs, len)
}

fn abi_get(offset: u64, buf: &mut [u8]) {
    unsafe { s_get(offset, &mut buf[0]) }
}

struct AbiStoreInner {
    data: [u8; 64 * 1024],
    written: usize,
    token: Token,
}

struct AbiStore {
    inner: UnsafeCell<AbiStoreInner>,
}

impl Fallible for AbiStore {
    type Error = core::convert::Infallible;
}

impl AbiStoreInner {
    fn get(&mut self, ident: &OffsetLen) -> &[u8] {
        let offset = ident.offset();
        let len = ident.len();
        let to_write = &mut self.data[self.written..][..len as usize];
        self.written += len as usize;
        abi_get(offset, to_write);
        to_write
    }

    fn return_token(&mut self, token: Token) {
        self.token.return_token(token)
    }
}

impl Store for AbiStore {
    type Identifier = OffsetLen;

    fn get(&self, ident: &OffsetLen) -> &[u8] {
        let inner = unsafe { &mut *self.inner.get() };
        inner.get(ident)
    }

    fn request_buffer(&self) -> TokenBuffer {
        todo!()
    }

    fn persist(&self) -> Result<(), ()> {
        Err(())
    }

    fn commit(&self, buffer: &mut TokenBuffer) -> Self::Identifier {
        let slice = buffer.written_bytes();
        assert!(slice.len() <= u16::MAX as usize);
        abi_put(slice)
    }

    fn extend(&self, buffer: &mut TokenBuffer) {
        // We can't
        ()
    }

    fn return_token(&self, token: Token) {
        let inner = unsafe { &mut *self.inner.get() };
        inner.return_token(token)
    }
}
