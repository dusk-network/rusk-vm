// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::VMError;

use canonical::Store;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub mod block_height;
pub mod debug;
pub mod gas;
pub mod panic;
pub mod query;
pub mod self_id;
pub mod store;
pub mod transact;

pub trait AbiCall<S>
where
    S: Store,
{
    const ARGUMENTS: &'static [ValueType];
    const RETURN: Option<ValueType>;

    fn call(
        context: &mut CallContext<S>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>>;
}
