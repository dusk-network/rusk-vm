// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! #Rusk-VM
//!
//! The main engine for executing WASM on the network state
#![warn(missing_docs)]
#![allow(unreachable_code)]

use std::{fmt, io};

use canonical::CanonError;
use failure::Fail;

mod call_context;
mod config;
mod contract;
mod gas;
mod ops;
mod resolver;
mod state;

pub use dusk_abi;

pub use call_context::StandardABI;
pub use contract::{Contract, ContractId};
pub use gas::{Gas, GasMeter};
pub use state::NetworkState;

#[derive(Fail)]
/// The errors that can happen while executing the VM
pub enum VMError {
    /// Invalid arguments in host call
    InvalidArguments,
    /// The contract panicked with message in `String`
    ContractPanic(String),
    /// Could not find WASM memory
    MemoryNotFound,
    /// Error during the instrumentation
    InstrumentationError(config::InstrumentationError),
    /// Invalid ABI Call
    InvalidABICall,
    /// Invalid Utf8
    InvalidUtf8,
    /// Invalid Public key
    InvalidEd25519PublicKey,
    /// Invalid Signature
    InvalidEd25519Signature,
    /// Contract returned, not an error per se, this is how contracts return.
    ContractReturn(i32, i32),
    /// Contract execution ran out of gas
    OutOfGas,
    /// Not enough funds for call
    NotEnoughFunds,
    /// Contract could not be found in the state
    UnknownContract,
    /// WASM threw an error
    WASMError(failure::Error),
    /// Wasmi trap triggered
    Trap(wasmi::Trap),
    /// Wasmi threw an error
    WasmiError(wasmi::Error),
    /// Input output error
    IOError(io::Error),
    /// Invalid WASM Module
    InvalidWASMModule,
    /// Error propagated from underlying store
    StoreError(CanonError),
}

impl From<io::Error> for VMError {
    fn from(e: io::Error) -> Self {
        VMError::IOError(e)
    }
}

impl From<wasmi::Error> for VMError {
    fn from(e: wasmi::Error) -> Self {
        VMError::WasmiError(e)
    }
}

impl From<wasmi::Trap> for VMError {
    fn from(e: wasmi::Trap) -> Self {
        VMError::Trap(e)
    }
}

impl From<config::InstrumentationError> for VMError {
    fn from(e: config::InstrumentationError) -> Self {
        VMError::InstrumentationError(e)
    }
}

// The generic From<CanonError> is not specific enough and conflicts with
// From<Self>.
impl VMError {
    /// Create a VMError from the associated stores
    pub fn from_store_error(err: CanonError) -> Self {
        VMError::StoreError(err)
    }
}

impl wasmi::HostError for VMError {}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VMError::InvalidArguments => write!(f, "Invalid arguments")?,
            VMError::ContractPanic(string) => {
                write!(f, "Contract panic \"{}\"", string)?
            }
            VMError::InvalidUtf8 => write!(f, "Invalid UTF-8")?,
            VMError::InvalidEd25519PublicKey => {
                write!(f, "Invalid Ed25519 Public Key")?
            }
            VMError::InvalidEd25519Signature => {
                write!(f, "Invalid Ed25519 Signature")?
            }
            VMError::ContractReturn(_, _) => write!(f, "Contract Return")?,
            VMError::OutOfGas => write!(f, "Out of Gas error")?,
            VMError::NotEnoughFunds => write!(f, "Not enough funds error")?,
            VMError::WASMError(e) => write!(f, "WASM Error ({:?})", e)?,
            VMError::MemoryNotFound => write!(f, "Memory not found")?,
            VMError::InvalidABICall => write!(f, "Invalid ABI Call")?,
            VMError::IOError(e) => write!(f, "Input/Output Error ({:?})", e)?,
            VMError::Trap(e) => write!(f, "Trap ({:?})", e)?,
            VMError::WasmiError(e) => write!(f, "WASMI Error ({:?})", e)?,
            VMError::UnknownContract => write!(f, "Unknown Contract")?,
            VMError::InvalidWASMModule => write!(f, "Invalid WASM module")?,
            VMError::StoreError(e) => write!(f, "Store error {:?}", e)?,
            VMError::InstrumentationError(e) => {
                write!(f, "Instrumentation error {:?}", e)?
            }
        }
        Ok(())
    }
}

impl fmt::Debug for VMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
