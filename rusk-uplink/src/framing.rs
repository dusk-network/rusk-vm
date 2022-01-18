

use rkyv::{Archive, Deserialize, Fallible, Serialize};
use rkyv::ser::serializers::BufferSerializer;
use rkyv::ser::Serializer;


pub struct EmptyStore;

impl Fallible for EmptyStore {
    type Error = core::convert::Infallible;
}


pub fn get_state_and_arg<S, P>(written_state: u32, written_data: u32, scratch: impl AsRef<[u8]>) -> (S, P)
    where S: Archive,
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
        archived_root::<P>(&scratch.as_ref()[written_state as usize..written_data as usize])
    };
    let arg: P = arg.deserialize(&mut store).unwrap();

    (state, arg)
}

pub fn get_state<S>(written_state: u32, scratch: impl AsRef<[u8]>) -> S
    where S: Archive,
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

pub fn t_return<'a, S, R>(state: &S, ret: &R, scratch: &'a mut [u8]) -> [u32; 2]
    where S: Serialize<BufferSerializer<&'a mut [u8]>>,
          R: Archive + Serialize<BufferSerializer<&'a mut [u8]>>,
{
    let mut ser = BufferSerializer::new(scratch);
    let state_len = ser.serialize_value(state).unwrap()
        + core::mem::size_of::<<S as Archive>::Archived>();

    let return_len = ser.serialize_value(ret).unwrap()
        + core::mem::size_of::<
        <R as Archive>::Archived,
    >();

    [state_len as u32, return_len as u32]
}

pub fn q_return<'a, R>(ret: &R, scratch: &'a mut [u8]) -> u32
    where R: Archive + Serialize<BufferSerializer<&'a mut [u8]>>
{
    let mut ser = BufferSerializer::new(scratch);
    let buffer_len = ser.serialize_value(ret).unwrap()
        + core::mem::size_of::<
        <R as Archive>::Archived,
    >();
    buffer_len as u32
}

#[macro_export]
macro_rules! query_state_arg_fun {
    ($fun_name:ident, $state_type:ty, $arg_type:ty) => (
        #[no_mangle]
        fn $fun_name(written_state: u32, written_data: u32) -> u32 {
            let (state, arg): ($state_type, $arg_type) = unsafe { get_state_and_arg(written_state, written_data, &SCRATCH) };

            let store =
                StoreContext::new(AbiStore::new(unsafe { &mut SCRATCH }));
            let res: <$arg_type as Query>::Return =
                state.execute(&arg, store);

            unsafe { q_return(&res, &mut SCRATCH) }
        }
    );
}
