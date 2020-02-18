use crate::host_fns::CallContext;
use crate::VMError;
use parking_lot::RwLock;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub trait ABICall: Send + Sync {
    fn call(
        &self,
        context: &mut CallContext,
        args: &RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError>;
    fn args(&self) -> &'static [ValueType];
    fn ret(&self) -> Option<ValueType>;
}

const REGISTERED_ABI_CALLS: RwLock<Vec<Box<dyn ABICall>>> = Default::default();

#[macro_export]
macro_rules! abi_call {
    ( $name:ident $arg_type:tt | $context:ident, $args: ident | $body:expr) => {
        #[derive(Clone, Copy)]
        struct $name;

        impl ABICall for $name {
            fn call(
                &self,
                $context: &mut CallContext,
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
