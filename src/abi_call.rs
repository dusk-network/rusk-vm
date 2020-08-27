// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use crate::call_context::CallContext;
use crate::VMError;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub trait ABICall<S>: Send + Sync {
    fn call(
        &self,
        context: &mut CallContext<S, H>,
        args: &RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError>;
    fn args(&self) -> &'static [ValueType];
    fn ret(&self) -> Option<ValueType>;
}

#[macro_export]
macro_rules! abi_call {
    ( $name:ident $arg_type:tt | $context:ident, $args: ident | $body:expr) => {
        #[derive(Clone, Copy)]
        struct $name;

        impl<S: Resolver> ABICall<S> for $name {
            fn call(
                &self,
                $context: &mut CallContext<S, H>,
                $args: &RuntimeArgs,
            ) -> Result<Option<RuntimeValue>, VMError> {
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
