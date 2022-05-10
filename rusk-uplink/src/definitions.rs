// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::fmt::Debug;

use bytecheck::CheckBytes;
use microkelvin::{OffsetLen, StoreRef, StoreSerializer};
use rkyv::{
    archived_root, check_archived_root,
    ser::Serializer,
    validation::{
        validators::{DefaultValidator, DefaultValidatorError},
        CheckArchiveError,
    },
    Archive, Deserialize, Serialize,
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
    CheckBytes,
)]
#[archive(as = "Self")]
pub struct ContractId([u8; 32]);

pub type StoreContext = StoreRef<OffsetLen>;

impl ContractId {
    /// Return a reserved contract id for host fn modules
    pub const fn reserved(id: u8) -> Self {
        let mut bytes = [0; 32];
        bytes[0] = id;
        ContractId(bytes)
    }

    /// Returns the contract id as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }

    /// Returns the contract id as a mutable slice
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }

    /// Returns a `ContractId` from an array of 32 bytes
    pub fn as_array(&self) -> [u8; 32] {
        self.0
    }

    /// Returns a `ContractId` from a mutable array of 32 bytes
    pub fn as_array_ref(&self) -> &[u8; 32] {
        &self.0
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
    fn execute(&self, q: Q, store: StoreContext) -> Q::Return;
}

pub trait Apply<T>
where
    T: Transaction,
{
    fn apply(&mut self, t: T, store: StoreContext) -> T::Return;
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
pub struct ReturnValue {
    data: Box<[u8]>,
    state: Box<[u8]>,
}

impl ReturnValue {
    pub fn new(result: impl AsRef<[u8]>) -> Self {
        let result = Box::from(result.as_ref());
        ReturnValue {
            data: result,
            state: Box::from([].as_ref()),
        }
    }

    pub fn with_state(
        result: impl AsRef<[u8]>,
        state: impl AsRef<[u8]>,
    ) -> Self {
        let result = Box::from(result.as_ref());
        let state = Box::from(state.as_ref());
        ReturnValue {
            data: result,
            state,
        }
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
        check_archived_root::<T>(&self.data[..])
    }

    pub fn cast_state<T>(&self) -> &T::Archived
    where
        T: Archive,
    {
        let state: &T::Archived =
            unsafe { archived_root::<T>(&self.state[..]) };
        state
    }

    pub fn cast_data<T>(&self) -> &T::Archived
    where
        T: Archive,
    {
        let data: &T::Archived = unsafe { archived_root::<T>(&self.data[..]) };
        data
    }

    pub fn data_len(&self) -> usize {
        self.data.len()
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }

    pub fn state_len(&self) -> usize {
        self.state.len()
    }

    pub fn state(&self) -> &[u8] {
        &self.state[..]
    }

    pub fn encode_lenghts(&self) -> u64 {
        ((self.data_len() as u64 + self.state_len() as u64) << 32)
            + self.state_len() as u64
    }
}

#[derive(Debug, Default)]
pub struct RawQuery<'a> {
    data: Vec<u8>,
    name: &'a str,
}

impl<'a> RawQuery<'a> {
    pub fn new<Q>(q: Q, store: &StoreRef<OffsetLen>) -> Self
    where
        Q: Query + Serialize<StoreSerializer<OffsetLen>>,
    {
        let mut ser = store.serializer();
        ser.serialize_value(&q).unwrap();
        RawQuery {
            data: ser.spill_bytes(|bytes| Vec::from(bytes)),
            name: Q::NAME,
        }
    }

    pub fn from<D: Into<Vec<u8>>>(data: D, name: &'a str) -> Self {
        Self {
            data: data.into(),
            name,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }
}

#[derive(Debug, Default)]
pub struct RawTransaction<'a> {
    data: Vec<u8>,
    name: &'a str,
}

impl<'a> RawTransaction<'a> {
    pub fn new<T>(q: T, store: &StoreRef<OffsetLen>) -> Self
    where
        T: Transaction + Serialize<StoreSerializer<OffsetLen>>,
    {
        let mut ser = store.serializer();
        ser.serialize_value(&q).unwrap();
        RawTransaction {
            data: ser.spill_bytes(|bytes| Vec::from(bytes)),
            name: T::NAME,
        }
    }

    pub fn from<D: Into<Vec<u8>>>(data: D, name: &'a str) -> Self {
        Self {
            data: data.into(),
            name,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }
}

#[derive(Debug, Default)]
pub struct RawEvent {
    data: Vec<u8>,
    name: String,
}

impl RawEvent {
    pub fn new<S, D>(name: S, data: D, store: &StoreRef<OffsetLen>) -> Self
    where
        S: Into<String>,
        D: Archive + Serialize<StoreSerializer<OffsetLen>>,
    {
        let mut ser = store.serializer();
        ser.serialize_value(&data).unwrap();
        RawEvent {
            data: ser.spill_bytes(|bytes| Vec::from(bytes)),
            name: name.into(),
        }
    }

    pub fn from<S, D: Into<Vec<u8>>>(name: S, data: D) -> Self
    where
        S: Into<String>,
        D: Into<Vec<u8>>,
    {
        Self {
            data: data.into(),
            name: name.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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
