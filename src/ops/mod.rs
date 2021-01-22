// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use crate::call_context::CallContext;
use crate::VMError;

use canonical::Store;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub mod debug;
pub mod gas;
pub mod panic;
pub mod query;
pub mod self_id;
pub mod store;
pub mod transact;

pub trait AbiCall<E, S>
where
    S: Store,
{
    const ARGUMENTS: &'static [ValueType];
    const RETURN: Option<ValueType>;

    fn call(
        context: &mut CallContext<E, S>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>>;
}
