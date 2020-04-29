use super::AbiCall;
use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use dusk_abi::{PodExt, H256};
use kelvin::ByteHash;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct CallContract;
pub struct CallContractOp;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for CallContract {
    const ARGUMENTS: &'static [ValueType] = &[
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
    ];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let target_ptr = args.get(0)? as usize;
        let amount_ptr = args.get(1)? as usize;
        let argument_ptr = args.get(2)? as usize;
        let argument_len = args.get(3)? as usize;
        let return_ptr = args.get(4)? as usize;
        let return_len = args.get(5)? as usize;

        let (target, amount) = context.memory(|m| {
            (
                H256::from_slice(&m[target_ptr..]),
                u128::from_slice(&m[amount_ptr..]),
            )
        });

        // First, transfer the amount
        if context.balance()? >= amount {
            *context.balance_mut()? -= amount;
            *context
                .state_mut()
                .get_contract_state_mut_or_default(&target)?
                .balance_mut() += amount;
        } else {
            // Return funding errors early
            return Err(VMError::NotEnoughFunds);
        }

        // Perform the call
        context.call(
            target,
            0,
            argument_ptr,
            argument_len,
            return_ptr,
            return_len,
        )
    }
}

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for CallContractOp {
    const ARGUMENTS: &'static [ValueType] = &[
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
        ValueType::I32,
    ];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let target_ptr = args.get(0)? as usize;
        let opcode = args.get(1)? as usize;
        let amount_ptr = args.get(2)? as usize;
        let argument_ptr = args.get(3)? as usize;
        let argument_len = args.get(4)? as usize;
        let return_ptr = args.get(5)? as usize;
        let return_len = args.get(6)? as usize;

        let (target, amount) = context.memory(|m| {
            (
                H256::from_slice(&m[target_ptr..]),
                u128::from_slice(&m[amount_ptr..]),
            )
        });

        // First, transfer the amount
        if context.balance()? >= amount {
            *context.balance_mut()? -= amount;
            *context
                .state_mut()
                .get_contract_state_mut_or_default(&target)?
                .balance_mut() += amount;
        } else {
            // Return funding errors early
            return Err(VMError::NotEnoughFunds);
        }

        // Perform the call
        context.call(
            target,
            opcode,
            argument_ptr,
            argument_len,
            return_ptr,
            return_len,
        )
    }
}
