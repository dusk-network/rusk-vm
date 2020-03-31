use super::AbiCall;
use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use dataview::Pod;
use dusk_abi::{CALL_DATA_SIZE, H256};
use kelvin::ByteHash;
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub struct CallContract;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for CallContract {
    const ARGUMENTS: &'static [ValueType] = &[
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
        let target_ofs = args.get(0)?;
        let amount_ofs = args.get(1)?;
        let data_ofs = args.get(2)?;
        let data_len = args.get(3)?;

        let mut call_buf = [0u8; CALL_DATA_SIZE];
        let mut target = H256::default();
        let mut amount = u64::default();

        context
            .memory()
            .with_direct_access::<Result<(), VMError>, _>(|a| {
                target
                    .as_bytes_mut()
                    .copy_from_slice(&a[target_ofs..target_ofs + 32]);
                amount
                    .as_bytes_mut()
                    .copy_from_slice(&a[amount_ofs..amount_ofs + 8]);
                call_buf[0..data_len]
                    .copy_from_slice(&a[data_ofs..data_ofs + data_len]);
                Ok(())
            })?;
        // assure sufficient funds are available
        if context.balance()? >= amount {
            *context.balance_mut()? -= amount;
            *context
                .state_mut()
                .get_contract_state_mut_or_default(&target)?
                .balance_mut() += amount;
        } else {
            panic!("not enough funds")
        }

        // if data_len > 0 {
        //     // write the return data back into memory
        //     context.memory().with_direct_access_mut(|a| {
        //         a[data_ofs..data_ofs + CALL_DATA_SIZE]
        //             .copy_from_slice(&return_buf)
        //     })
        // }

        Ok(None)
    }
}
