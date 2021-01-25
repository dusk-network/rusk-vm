// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::call_context::CallContext;
use crate::VMError;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub trait ABICall<E>: Send + Sync {
    fn call(
        &self,
        context: &mut CallContext<E, S>,
        args: &RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError<S>>;
    fn args(&self) -> &'static [ValueType];
    fn ret(&self) -> Option<ValueType>;
}

#[macro_export]
macro_rules! abi_call {
    ( $name:ident $arg_type:tt | $context:ident, $args: ident | $body:expr) => {
        #[derive(Clone, Copy)]
        struct $name;

        impl<Re: Resolver> ABICall<Re> for $name {
            fn call(
                &self,
                $context: &mut CallContext<Re, S>,
                $args: &RuntimeArgs,
            ) -> Result<Option<RuntimeValue>, VMError<S>> {
                $body
            }

            fn args(&self) -> &'static [ValueType] {
                &$arg_type
            }

            fn ret(&self) -> Option<ValueType> {
                None
            }
        }
    };
}
