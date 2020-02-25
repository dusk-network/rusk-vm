use crate::host_fns::CallContext;
use crate::VMError;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub trait ABICall<S>: Send + Sync {
    fn call(
        &self,
        context: &mut CallContext<S>,
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
                $context: &mut CallContext<S>,
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
