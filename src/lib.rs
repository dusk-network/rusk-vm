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

mod call_context;
mod compiler;
mod contract;
mod env;
mod gas;
mod memory;
mod module_config;
mod ops;
mod resolver;
mod state;

pub use dusk_abi;

pub use contract::{Contract, ContractId};
pub use gas::{Gas, GasMeter};
pub use state::NetworkState;

use thiserror::Error;
use wasmer_vm::TrapCode;

#[derive(Error)]
//#[derive(Fail)]
/// The errors that can happen while executing the VM
pub enum VMError {
    /// Invalid arguments in host call
    InvalidArguments,
    /// The contract panicked with message in `String`
    ContractPanic(String),
    /// Could not find WASM memory
    MemoryNotFound,
    /// Error during the instrumentalization
    InstrumentationError(module_config::InstrumentationError),
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
    /// Input output error
    IOError(io::Error),
    /// Invalid WASM Module
    InvalidWASMModule,
    /// Error propagated from underlying store
    StoreError(CanonError),
    /// Serialization error from the state persistence mechanism
    PersistenceSerializationError(CanonError),
    /// Other error from the state persistence mechanism
    PersistenceError(String),
    /// WASMER export error
    WasmerExportError(wasmer::ExportError),
    /// WASMER runtime error
    WasmerRuntimeError(wasmer::RuntimeError),
    /// WASMER compile error
    WasmerCompileError(wasmer::CompileError),
    /// WASMER trap
    WasmerTrap(TrapCode),
    /// WASMER instantiation error
    WasmerInstantiationError(wasmer::InstantiationError),
}

impl From<io::Error> for VMError {
    fn from(e: io::Error) -> Self {
        VMError::IOError(e)
    }
}

impl From<module_config::InstrumentationError> for VMError {
    fn from(e: module_config::InstrumentationError) -> Self {
        VMError::InstrumentationError(e)
    }
}

impl From<wasmer::InstantiationError> for VMError {
    fn from(e: wasmer::InstantiationError) -> Self {
        VMError::WasmerInstantiationError(e)
    }
}

impl From<wasmer::ExportError> for VMError {
    fn from(e: wasmer::ExportError) -> Self {
        VMError::WasmerExportError(e)
    }
}

impl From<wasmer::CompileError> for VMError {
    fn from(e: wasmer::CompileError) -> Self {
        VMError::WasmerCompileError(e)
    }
}

impl From<wasmer::RuntimeError> for VMError {
    fn from(e: wasmer::RuntimeError) -> Self {
        let runtime_error = e.clone();
        match e.to_trap() {
            Some(trap_code) => VMError::WasmerTrap(trap_code),
            _ => VMError::WasmerRuntimeError(runtime_error),
        }
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
            VMError::UnknownContract => write!(f, "Unknown Contract")?,
            VMError::InvalidWASMModule => write!(f, "Invalid WASM module")?,
            VMError::StoreError(e) => write!(f, "Store error {:?}", e)?,
            VMError::InstrumentationError(e) => {
                write!(f, "Instrumentalization error {:?}", e)?
            }
            VMError::PersistenceSerializationError(e) => {
                write!(f, "Persistence serialization error {:?}", e)?
            }
            VMError::PersistenceError(string) => {
                write!(f, "Persistence error \"{}\"", string)?
            }
            VMError::WasmerExportError(e) => match e {
                wasmer::ExportError::IncompatibleType => {
                    write!(f, "WASMER Export Error - incompatible export type")?
                }
                wasmer::ExportError::Missing(s) => {
                    write!(f, "WASMER Export Error - missing: \"{}\"", s)?
                }
            },
            VMError::WasmerRuntimeError(e) => {
                write!(f, "WASMER Runtime Error {:?}", e)?
            }
            VMError::WasmerTrap(e) => write!(f, "WASMER Trap ({:?})", e)?,
            VMError::WasmerInstantiationError(e) => {
                write!(f, "WASMER Instantiation Error ({:?})", e)?
            }
            VMError::WasmerCompileError(e) => {
                write!(f, "WASMER Compile Error {:?}", e)?
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

/// Definition of the cost schedule and other parameterizations for wasm vm.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq)]
pub struct Schedule {
    /// Version of the schedule.
    pub version: u32,

    /// Cost of putting a byte of code into storage.
    pub put_code_per_byte_cost: Gas,

    /// Gas cost of a growing memory by single page.
    pub grow_mem_cost: Gas,

    /// Gas cost of a regular operation.
    pub regular_op_cost: Gas,

    /// Gas cost per one byte returned.
    pub return_data_per_byte_cost: Gas,

    /// Gas cost to deposit an event; the per-byte portion.
    pub event_data_per_byte_cost: Gas,

    /// Gas cost to deposit an event; the cost per topic.
    pub event_per_topic_cost: Gas,

    /// Gas cost to deposit an event; the base.
    pub event_base_cost: Gas,

    /// Base gas cost to call into a contract.
    pub call_base_cost: Gas,

    /// Base gas cost to instantiate a contract.
    pub instantiate_base_cost: Gas,

    /// Gas cost per one byte read from the sandbox memory.
    pub sandbox_data_read_cost: Gas,

    /// Gas cost per one byte written to the sandbox memory.
    pub sandbox_data_write_cost: Gas,

    /// The maximum number of topics supported by an event.
    pub max_event_topics: u32,

    /// Maximum allowed stack height.
    ///
    /// See https://wiki.parity.io/WebAssembly-StackHeight to find out
    /// how the stack frame cost is calculated.
    pub max_stack_height: u32,

    /// Maximum number of memory pages allowed for a contract.
    pub max_memory_pages: u32,

    /// Maximum allowed size of a declared table.
    pub max_table_size: u32,

    /// Whether the `ext_println` function is allowed to be used contracts.
    /// MUST only be enabled for `dev` chains, NOT for production chains
    pub enable_println: bool,

    /// The maximum length of a subject used for PRNG generation.
    pub max_subject_len: u32,
}

impl Default for Schedule {
    fn default() -> Schedule {
        Schedule {
            version: 0,
            put_code_per_byte_cost: 1,
            grow_mem_cost: 1,
            regular_op_cost: 1,
            return_data_per_byte_cost: 1,
            event_data_per_byte_cost: 1,
            event_per_topic_cost: 1,
            event_base_cost: 1,
            call_base_cost: 135,
            instantiate_base_cost: 175,
            sandbox_data_read_cost: 1,
            sandbox_data_write_cost: 1,
            max_event_topics: 4,
            max_stack_height: 64 * 1024,
            max_memory_pages: 16,
            max_table_size: 16 * 1024,
            enable_println: false,
            max_subject_len: 32,
        }
    }
}
