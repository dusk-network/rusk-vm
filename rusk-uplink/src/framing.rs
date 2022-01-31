use crate::StoreContext;
use microkelvin::{OffsetLen, StoreSerializer};
use rkyv::ser::Serializer;
use rkyv::{Archive, Deserialize, Serialize};

pub fn get_state_arg<S, P>(
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

pub fn q_return<R>(ret: &R, store: StoreContext) -> u32
where
    R: Archive + Serialize<StoreSerializer<OffsetLen>>,
{
    let mut ser = store.serializer();
    let buffer_len = ser.serialize_value(ret).unwrap()
        + core::mem::size_of::<<R as Archive>::Archived>();
    buffer_len as u32
}

pub fn t_return<S, R>(
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
