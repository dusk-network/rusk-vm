// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::gas;
use crate::modules;

use canonical::CanonError;
use dusk_abi::ContractId;
use std::io;
use thiserror::Error;
use wasmer_vm::TrapCode;

#[derive(Error, Debug)]
/// The errors that can happen while executing the VM
pub enum VMError {
    /// The Stack is empty
    #[error("The Stack is empty")]
    EmptyStack,
    /// The Contract Panicked
    #[error("The contract {0} panicked with message: {1}")]
    ContractPanic(ContractId, String),
    /// Instrumentation Error
    #[error(transparent)]
    InstrumentationError(#[from] modules::InstrumentationError),
    /// Invalid UTF-8
    #[error("Invalid UTF-8")]
    InvalidUtf8,
    /// Contract execution ran out of gas
    #[error("Contract execution ran out of gas")]
    OutOfGas,
    /// Contract could not be found in the state
    #[error("Contract {0} could not be found in the state")]
    UnknownContract(ContractId),
    /// Input / Output error
    #[error("Input / Output error")]
    IOError(#[from] io::Error),
    /// Error propagated from underlying store
    #[error("Error propagated from underlying store")]
    StoreError(CanonError),
    /// WASMER export error
    #[error(transparent)]
    WasmerExportError(#[from] wasmer::ExportError),
    /// WASMER runtime error
    #[error(transparent)]
    WasmerRuntimeError(wasmer::RuntimeError),
    /// WASMER  compile error
    #[error(transparent)]
    WasmerCompileError(#[from] wasmer::CompileError),
    /// WASMER instantiation error
    #[error(transparent)]
    WasmerInstantiationError(#[from] wasmer::InstantiationError),
    /// WASMER trap
    #[error("WASMER trap")]
    WasmerTrap(TrapCode),
}

impl From<gas::GasError> for VMError {
    fn from(_: gas::GasError) -> Self {
        // Currently the only gas error is `GasLimitExceeded`
        VMError::OutOfGas
    }
}

impl From<wasmer::RuntimeError> for VMError {
    fn from(e: wasmer::RuntimeError) -> Self {
        // NOTE: Do not clone before downcasting!
        // `RuntimeError::downcast` calls `Arc::try_unwrap` which will fail to
        // downcast if there is more than one reference to the `Arc`.
        let e = match e.downcast::<VMError>() {
            Ok(vm_error) => return vm_error,
            Err(err) => err,
        };

        match e.clone().to_trap() {
            Some(trap_code) => VMError::WasmerTrap(trap_code),
            None => VMError::WasmerRuntimeError(e),
        }
    }
}

impl From<CanonError> for VMError {
    fn from(e: CanonError) -> Self {
        VMError::StoreError(e)
    }
}
