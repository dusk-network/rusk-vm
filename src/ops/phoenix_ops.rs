use super::AbiCall;
use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;
use std::convert::{TryFrom, TryInto};
use std::env;
use std::path::Path;

use kelvin::ByteHash;
use phoenix::{
    db, utils, zk, BlsScalar, NoteGenerator, NoteVariant, PublicKey,
    Transaction, TransactionInput, TransactionOutput, TransparentNote,
};
use phoenix_abi::{Input, Note, Proof};
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

const SUCCESS: Result<Option<RuntimeValue>, VMError> =
    Ok(Some(RuntimeValue::I32(1)));

const FAIL: Result<Option<RuntimeValue>, VMError> =
    Ok(Some(RuntimeValue::I32(0)));

pub struct PhoenixStore;

#[inline]
fn has_value(bytes: &[u8]) -> bool {
    !bytes.iter().all(|b| *b == 0)
}

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for PhoenixStore {
    const ARGUMENTS: &'static [ValueType] =
        &[ValueType::I32, ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = Some(ValueType::I32);

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        todo!("port phoenixstore");
    }
}

pub struct PhoenixVerify;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for PhoenixVerify {
    const ARGUMENTS: &'static [ValueType] =
        &[ValueType::I32, ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = Some(ValueType::I32);

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        todo!("port phoenixverify");
    }
}

pub struct PhoenixCredit;

// TODO: note credited is always transparent
// we should give the option (or maybe a separate host function)
// to make obfuscated ones
impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for PhoenixCredit {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = Some(ValueType::I32);

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        todo!("port phoenixcredit")
    }
}
