use crate::StoreContext;
use microkelvin::{OffsetLen, StoreSerializer};
use rkyv::ser::serializers::BufferSerializer;
use rkyv::ser::Serializer;
use rkyv::{Archive, Deserialize, Fallible, Serialize};

pub struct EmptyStore;

impl Fallible for EmptyStore {
    type Error = core::convert::Infallible;
}

pub fn get_state_arg<S, P>(
    written_state: u32,
    written_data: u32,
    scratch: impl AsRef<[u8]>,
) -> (S, P)
where
    S: Archive,
    <S as Archive>::Archived: Deserialize<S, EmptyStore>,
    P: Archive,
    <P as Archive>::Archived: Deserialize<P, EmptyStore>,
{
    use rkyv::archived_root;

    let mut store = EmptyStore;

    let state = unsafe {
        archived_root::<S>(&scratch.as_ref()[..written_state as usize])
    };
    let state: S = state.deserialize(&mut store).unwrap();
    let arg = unsafe {
        archived_root::<P>(
            &scratch.as_ref()[written_state as usize..written_data as usize],
        )
    };
    let arg: P = arg.deserialize(&mut store).unwrap();

    (state, arg)
}

pub fn get_state_arg_store<S, P>(
    written_state: u32,
    written_data: u32,
    scratch: impl AsRef<[u8]>,
    store: StoreContext,
) -> (S, P)
where
    S: Archive,
    <S as Archive>::Archived: Deserialize<S, StoreContext>,
    P: Archive,
    <P as Archive>::Archived: Deserialize<P, StoreContext>,
{
    use rkyv::archived_root;

    let state = unsafe {
        archived_root::<S>(&scratch.as_ref()[..written_state as usize])
    };
    let state: S = state.deserialize(&mut store.clone()).unwrap();
    let arg = unsafe {
        archived_root::<P>(
            &scratch.as_ref()[written_state as usize..written_data as usize],
        )
    };
    let arg: P = arg.deserialize(&mut store.clone()).unwrap();

    (state, arg)
}

pub fn get_state<S>(written_state: u32, scratch: impl AsRef<[u8]>) -> S
where
    S: Archive,
    <S as Archive>::Archived: Deserialize<S, EmptyStore>,
{
    use rkyv::archived_root;

    let mut store = EmptyStore;

    let state = unsafe {
        archived_root::<S>(&scratch.as_ref()[..written_state as usize])
    };
    let state: S = state.deserialize(&mut store).unwrap();

    state
}

pub fn q_return<'a, R>(ret: &R, scratch: &'a mut [u8]) -> u32
where
    R: Archive + Serialize<BufferSerializer<&'a mut [u8]>>,
{
    let mut ser = BufferSerializer::new(scratch);
    let buffer_len = ser.serialize_value(ret).unwrap()
        + core::mem::size_of::<<R as Archive>::Archived>();
    buffer_len as u32
}

pub fn q_return_store_ser<R>(ret: &R, store: StoreContext) -> u32
where
    R: Archive + Serialize<StoreSerializer<OffsetLen>>,
{
    let mut ser = store.serializer();
    let buffer_len = ser.serialize_value(ret).unwrap()
        + core::mem::size_of::<<R as Archive>::Archived>();
    buffer_len as u32
}

pub fn t_return<'a, S, R>(state: &S, ret: &R, scratch: &'a mut [u8]) -> [u32; 2]
where
    S: Serialize<BufferSerializer<&'a mut [u8]>>,
    R: Archive + Serialize<BufferSerializer<&'a mut [u8]>>,
{
    let mut ser = BufferSerializer::new(scratch);
    let state_len = ser.serialize_value(state).unwrap()
        + core::mem::size_of::<<S as Archive>::Archived>();

    let return_len = ser.serialize_value(ret).unwrap()
        + core::mem::size_of::<<R as Archive>::Archived>();

    [state_len as u32, return_len as u32]
}

pub fn t_return_store_ser<S, R>(
    state: &S,
    ret: &R,
    store: StoreContext,
) -> [u32; 2]
where
    S: Serialize<StoreSerializer<OffsetLen>>,
    R: Archive + Serialize<StoreSerializer<OffsetLen>>,
{
    let mut ser = store.serializer();
    let state_len = ser.serialize_value(state).unwrap()
        + core::mem::size_of::<<S as Archive>::Archived>();

    let return_len = ser.serialize_value(ret).unwrap()
        + core::mem::size_of::<<R as Archive>::Archived>();

    [state_len as u32, return_len as u32]
}

#[macro_export]
macro_rules! framing_imports {
    () => {
        use rusk_uplink::{
            get_state, get_state_arg, get_state_arg_store, q_return,
            q_return_store_ser, q_handler,
            q_handler_store_ser, t_return, t_return_store_ser,
            t_handler, t_handler_store_ser,
            AbiStore, scratch_memory
        };
    };
}

#[macro_export]
macro_rules! q_handler {
    ($fun_name:ident, $state_type:ty, $arg_type:ty) => {
        #[no_mangle]
        fn $fun_name(written_state: u32, written_data: u32) -> u32 {
            let (state, arg): ($state_type, $arg_type) = unsafe {
                get_state_arg(written_state, written_data, &SCRATCH)
            };

            let store =
                StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));
            let res: <$arg_type as Query>::Return = state.execute(arg, store);

            unsafe { q_return(&res, &mut SCRATCH) }
        }
    };
}

#[macro_export]
macro_rules! q_handler_store_ser {
    ($fun_name:ident, $state_type:ty, $arg_type:ty) => {
        #[no_mangle]
        fn $fun_name(written_state: u32, written_data: u32) -> u32 {
            let store =
                StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));
            let (state, arg): ($state_type, $arg_type) = unsafe {
                get_state_arg_store(
                    written_state,
                    written_data,
                    &SCRATCH,
                    store.clone(),
                )
            };

            let res: <$arg_type as Query>::Return =
                state.execute(arg, store.clone());

            unsafe { q_return_store_ser(&res, store) }
        }
    };
}

#[macro_export]
macro_rules! t_handler {
    ($fun_name:ident, $state_type:ty, $arg_type:ty) => {
        #[no_mangle]
        fn $fun_name(written_state: u32, written_data: u32) -> [u32; 2] {
            let (mut state, arg): ($state_type, $arg_type) = unsafe {
                get_state_arg(written_state, written_data, &SCRATCH)
            };

            let store =
                StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));
            let res: <$arg_type as Transaction>::Return =
                state.apply(arg, store);

            unsafe { t_return(&state, &res, &mut SCRATCH) }
        }
    };
}

#[macro_export]
macro_rules! t_handler_store_ser {
    ($fun_name:ident, $state_type:ty, $arg_type:ty) => {
        #[no_mangle]
        fn $fun_name(written_state: u32, written_data: u32) -> [u32; 2] {
            let store =
                StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));
            let (mut state, arg): ($state_type, $arg_type) = unsafe {
                get_state_arg_store(
                    written_state,
                    written_data,
                    &SCRATCH,
                    store.clone(),
                )
            };

            let res: <$arg_type as Transaction>::Return =
                state.apply(arg, store.clone());

            unsafe { t_return_store_ser(&state, &res, store) }
        }
    };
}

#[macro_export]
macro_rules! scratch_memory {
    ($sz:expr) => {
        #[no_mangle]
        static mut SCRATCH: [u8; $sz] = [0u8; $sz];
    }
}
