use microkelvin::{Ident, Offset, Storage, Store, Stored};
use rkyv::{ser::serializers::AllocSerializer, ser::Serializer, Fallible};

pub struct AbiStorage(AllocSerializer<1024>);

impl AbiStorage {
    fn new() -> Self {
        AbiStorage(Default::default())
    }
}

#[derive(Clone)]
pub struct AbiStore;

impl Fallible for AbiStore {
    type Error = core::convert::Infallible;
}

impl Fallible for AbiStorage {
    type Error = core::convert::Infallible;
}

impl Serializer for AbiStorage {
    fn pos(&self) -> usize {
        self.0.pos()
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), <Self as Fallible>::Error> {
        let _ = self.0.write(bytes);
        Ok(())
    }
}

extern "C" {
    fn s_put(slice: &u8, len: u32) -> u64;
    fn s_get(offset: u64) -> &'static ();
}

impl Storage<Offset> for AbiStorage {
    fn put<T>(&mut self, t: &T) -> Offset
    where
        T: rkyv::Serialize<Self>,
    {
        self.serialize_value(t).unwrap();
        let serializer = core::mem::take(&mut self.0);
        let bytes = &serializer.into_serializer().into_inner()[..];
        let put_ofs = unsafe { s_put(&bytes[0], bytes.len() as u32) };
        Offset(put_ofs)
    }

    fn get<T>(&self, id: &Offset) -> &T::Archived
    where
        T: rkyv::Archive,
    {
        let ofs = id.0;
        unsafe {
            let ptr: &() = s_get(ofs);
            core::mem::transmute(ptr)
        }
    }
}

impl Store for AbiStore {
    type Identifier = Offset;

    type Storage = AbiStorage;

    fn put<T>(&self, t: &T) -> Stored<T, Self>
    where
        T: rkyv::Serialize<Self::Storage>,
    {
        let mut storage = AbiStorage::new();
        Stored::new(self.clone(), Ident::new(storage.put::<T>(t)))
    }

    fn get_raw<'a, T>(
        &'a self,
        id: &Ident<Self::Identifier, T>,
    ) -> &'a T::Archived
    where
        T: rkyv::Archive,
    {
        let storage = AbiStorage::new();
        let reference = storage.get::<T>(&id.erase());
        let extended: &'a T::Archived =
            unsafe { core::mem::transmute(reference) };
        extended
    }
}
