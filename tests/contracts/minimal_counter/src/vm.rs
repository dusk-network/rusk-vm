use rkyv::Archive;

/// Temporary file defining some vm interface types

pub trait Method {
    const NAME: &'static str;
    type Return;
}

pub trait Apply<T: Method>
where
    Self: Archive,
    T: Archive,
{
    fn apply(&mut self, t: &T::Archived) -> T::Return;
}

pub trait Query<Q: Method>
where
    Self: Archive,
    Q: Archive,
{
    fn query(archived: &Self::Archived, q: &Q::Archived) -> Q::Return;
}

pub struct Portal;
