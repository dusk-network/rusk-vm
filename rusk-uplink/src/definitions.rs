use core::fmt::Debug;

use bytecheck::CheckBytes;
use rkyv::{
    check_archived_root,
    ser::{serializers::AllocSerializer, Serializer},
    validation::{
        validators::{DefaultValidator, DefaultValidatorError},
        CheckArchiveError,
    },
    AlignedVec, Archive, Deserialize, Serialize,
};

#[derive(
    PartialEq,
    Eq,
    Hash,
    Clone,
    Copy,
    Debug,
    Default,
    Archive,
    Serialize,
    Deserialize,
)]
#[archive(as = "Self")]
pub struct ContractId([u8; 32]);

impl ContractId {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

impl<B> From<B> for ContractId
where
    B: AsRef<[u8]>,
{
    fn from(b: B) -> Self {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(b.as_ref());
        ContractId(bytes)
    }
}

pub trait Execute<Q>
where
    Q: Query,
{
    fn execute(&self, q: &Q) -> Q::Return;
}

pub trait Apply<T>
where
    T: Transaction,
{
    fn apply(&mut self, t: &T) -> T::Return;
}

pub trait Query: Archive {
    const NAME: &'static str;

    type Return;
}

pub trait Transaction: Archive {
    const NAME: &'static str;

    type Return;
}

#[derive(Debug, Default, Archive, Serialize, Deserialize)]
pub struct ContractState(Vec<u8>);

impl ContractState {
    pub fn new(v: Vec<u8>) -> Self {
        Self(v)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

pub trait HostModule {
    fn execute(&self);

    fn module_id(&self) -> ContractId;
}

// TODO, use borrowed bytes here?
#[derive(Debug, Default)]
pub struct ReturnValue(pub Vec<u8>);

impl ReturnValue {
    pub fn new<V: Into<Vec<u8>>>(vec: V) -> Self {
        ReturnValue(vec.into())
    }

    pub fn cast<'a, T>(
        &'a self,
    ) -> Result<
        &'a T::Archived,
        CheckArchiveError<
            <T::Archived as CheckBytes<DefaultValidator<'a>>>::Error,
            DefaultValidatorError,
        >,
    >
    where
        T: Archive,
        T::Archived: CheckBytes<DefaultValidator<'a>>,
    {
        check_archived_root::<T>(&self.0[..])
    }
}

#[derive(Debug, Default)]
pub struct RawQuery<'a> {
    data: AlignedVec,
    name: &'a str,
}

impl<'a> RawQuery<'a> {
    pub fn new<Q>(q: Q) -> Self
    where
        Q: Query + Serialize<AllocSerializer<1024>>,
    {
        let mut ser = AllocSerializer::default();
        ser.serialize_value(&q).unwrap();
        RawQuery {
            data: ser.into_serializer().into_inner(),
            name: Q::NAME,
        }
    }

    pub fn from(data: AlignedVec, name: &'a str) -> Self {
        Self { data, name }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }
}

#[derive(Debug, Default)]
pub struct RawTransaction {
    data: AlignedVec,
    name: &'static str,
}

impl RawTransaction {
    pub fn new<T>(q: T) -> Self
    where
        T: Transaction + Serialize<AllocSerializer<1024>>,
    {
        let mut ser = AllocSerializer::default();
        ser.serialize_value(&q).unwrap();
        RawTransaction {
            data: ser.into_serializer().into_inner(),
            name: T::NAME,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }
}

// todo! find better way
#[derive(Debug)]
pub enum ArchiveError {
    ArchiveValidationError,
}
