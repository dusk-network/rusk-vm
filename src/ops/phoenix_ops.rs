use super::AbiCall;
use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use dusk_abi::encoding;
use kelvin::ByteHash;
use phoenix::{db, Transaction, TransactionItem};
use phoenix_abi::{
    types::{
        MAX_NOTES_PER_TRANSACTION, MAX_NULLIFIERS_PER_TRANSACTION, NOTE_SIZE,
        NULLIFIER_SIZE,
    },
    Note, Nullifier,
};
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub const DB_PATH: &'static str = "/tmp/rusk-vm-demo";

pub struct PhoenixStore;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for PhoenixStore {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = Some(ValueType::I32);

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let nullifiers_ptr = args.get(0)?;
        let notes_ptr = args.get(1)?;

        context
            .top()
            .memory
            .with_direct_access_mut::<Result<Option<RuntimeValue>, VMError>, _>(
                |a| {
                    let nullifiers_buf = &a[nullifiers_ptr
                        ..nullifiers_ptr
                            + (MAX_NULLIFIERS_PER_TRANSACTION
                                * NULLIFIER_SIZE)];
                    let nullifiers: Result<
                        Vec<TransactionItem>,
                        fermion::Error,
                    > = nullifiers_buf
                        .chunks(NULLIFIER_SIZE)
                        .map(|bytes| {
                            let nullifier: Nullifier = encoding::decode(bytes)?;
                            let mut item = TransactionItem::default();
                            item.set_nullifier(nullifier.into());
                            Ok(item)
                        })
                        .collect();
                    let mut nullifiers = nullifiers.unwrap();

                    let notes_buf = &a[notes_ptr
                        ..notes_ptr + (MAX_NOTES_PER_TRANSACTION * NOTE_SIZE)];

                    let notes: Result<Vec<TransactionItem>, fermion::Error> =
                        notes_buf
                            .chunks(NOTE_SIZE)
                            .map(|bytes| {
                                let note: Note = encoding::decode(bytes)?;
                                Ok(TransactionItem::from(note))
                            })
                            .collect();
                    let mut notes = notes.unwrap();

                    let items = nullifiers.drain(..).chain(notes.drain(..));

                    // TODO: decode proof and include it in the tx

                    let mut tx = Transaction::default();

                    items.for_each(|item| tx.push(item));

                    match db::store(DB_PATH, &tx) {
                        Ok(_) => Ok(Some(RuntimeValue::I32(1))),
                        Err(_) => Ok(Some(RuntimeValue::I32(0))),
                    }
                },
            )
    }
}

pub struct PhoenixVerify;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for PhoenixVerify {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = Some(ValueType::I32);

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let nullifiers_ptr = args.get(0)?;
        let notes_ptr = args.get(1)?;

        context
            .top()
            .memory
            .with_direct_access_mut::<Result<Option<RuntimeValue>, VMError>, _>(
                |a| {
                    let nullifiers_buf = &a[nullifiers_ptr
                        ..nullifiers_ptr
                            + (MAX_NULLIFIERS_PER_TRANSACTION
                                * NULLIFIER_SIZE)];
                    let nullifiers: Result<
                        Vec<TransactionItem>,
                        fermion::Error,
                    > = nullifiers_buf
                        .chunks(NULLIFIER_SIZE)
                        .map(|bytes| {
                            let nullifier: Nullifier = encoding::decode(bytes)?;
                            let mut item = TransactionItem::default();
                            item.set_nullifier(nullifier.into());
                            Ok(item)
                        })
                        .collect();
                    let mut nullifiers = nullifiers.unwrap();

                    let notes_buf = &a[notes_ptr
                        ..notes_ptr + (MAX_NOTES_PER_TRANSACTION * NOTE_SIZE)];

                    let notes: Result<Vec<TransactionItem>, fermion::Error> =
                        notes_buf
                            .chunks(NOTE_SIZE)
                            .map(|bytes| {
                                let note: Note = encoding::decode(bytes)?;
                                Ok(TransactionItem::from(note))
                            })
                            .collect();
                    let mut notes = notes.unwrap();

                    let items = nullifiers.drain(..).chain(notes.drain(..));

                    // TODO: decode proof and include it in the tx

                    let mut tx = Transaction::default();

                    items.for_each(|item| tx.push(item));

                    match tx.verify() {
                        Ok(_) => Ok(Some(RuntimeValue::I32(1))),
                        Err(_) => Ok(Some(RuntimeValue::I32(0))),
                    }
                },
            )
    }
}
