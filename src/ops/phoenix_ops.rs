use super::AbiCall;
use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use dusk_abi::encoding;
use kelvin::ByteHash;
use phoenix::{db, Transaction, TransactionItem};
use phoenix_abi::{Note, Nullifier};
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub const DB_PATH: &str = "/tmp/rusk-vm-demo";

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
                        ..nullifiers_ptr + (Nullifier::MAX * Nullifier::SIZE)];
                    let nullifiers: Result<
                        Vec<TransactionItem>,
                        fermion::Error,
                    > = nullifiers_buf
                        .chunks(Nullifier::SIZE)
                        .map(|bytes| {
                            let nullifier: Nullifier = encoding::decode(bytes)?;
                            let mut item = TransactionItem::default();
                            item.set_nullifier(nullifier.into());
                            Ok(item)
                        })
                        .collect();
                    let mut nullifiers = nullifiers.unwrap();

                    let notes_buf =
                        &a[notes_ptr..notes_ptr + (Note::MAX * Note::SIZE)];

                    let notes: Result<Vec<TransactionItem>, fermion::Error> =
                        notes_buf
                            .chunks(Note::SIZE)
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
                        ..nullifiers_ptr + (Nullifier::MAX * Nullifier::SIZE)];
                    let nullifiers: Result<
                        Vec<TransactionItem>,
                        fermion::Error,
                    > = nullifiers_buf
                        .chunks(Nullifier::SIZE)
                        .map(|bytes| {
                            let nullifier: Nullifier = encoding::decode(bytes)?;
                            let mut item = TransactionItem::default();
                            item.set_nullifier(nullifier.into());
                            Ok(item)
                        })
                        .collect();
                    let mut nullifiers = nullifiers.unwrap();

                    let notes_buf =
                        &a[notes_ptr..notes_ptr + (Note::MAX * Note::SIZE)];

                    let notes: Result<Vec<TransactionItem>, fermion::Error> =
                        notes_buf
                            .chunks(Note::SIZE)
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

pub struct PhoenixCredit;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for PhoenixCredit {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = Some(ValueType::I32);

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let reward = args.get(0)?;
        let pk_ptr = args.get(1)?;

        context
            .top()
            .memory
            .with_direct_access_mut::<Result<Option<RuntimeValue>, VMError>, _>(
                |a| {
                    let pk = &a[pk_ptr..pk_ptr + 32];

                    let (output, _) = TransparentNote::output(, reward);

                    let mut tx = Transaction::default();
                    tx.push(output);

                    match db::store(DB_PATH, &tx) {
                        Ok(_) => Ok(Some(RuntimeValue::I32(1))),
                        Err(_) => Ok(Some(RuntimeValue::I32(0))),
                    }
                },
            )
    }
}
