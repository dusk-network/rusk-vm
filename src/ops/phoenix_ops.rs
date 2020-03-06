use super::AbiCall;
use crate::host_fns::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use dusk_abi::encoding;
use kelvin::ByteHash;
use phoenix::{db, Transaction, TransactionItem};
use phoenix_abi::{Item, ITEM_SIZE, MAX_NOTES_PER_TRANSACTION};
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub const DB_PATH: &'static str = "/tmp/rusk-vm-demo";

pub struct PhoenixStore;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for PhoenixStore {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let items_ptr = args.get(0)?;

        context
            .top()
            .memory
            .with_direct_access_mut::<Result<Option<RuntimeValue>, VMError>, _>(
                |a| {
                    let items_buf = &a[items_ptr
                        ..items_ptr + (MAX_NOTES_PER_TRANSACTION * ITEM_SIZE)];

                    let items: Result<Vec<TransactionItem>, fermion::Error> =
                        items_buf
                            .chunks(ITEM_SIZE)
                            .map(|bytes| {
                                let item: Item = encoding::decode(bytes)?;
                                Ok(TransactionItem::from(item))
                            })
                            .collect();

                    // let mut proof_buf = [0u8; PROOF_SIZE];
                    // proof_buf
                    //     .copy_from_slice(&a[proof_ptr..proof_ptr + PROOF_SIZE]);

                    // match R1CSProof::from_bytes(&proof_buf[..]) {
                    //     Ok(proof) => {
                    let mut tx = Transaction::default();
                    // tx.set_r1cs(proof);

                    for item in items.unwrap() {
                        tx.push(item);
                    }

                    match db::store(DB_PATH, &tx) {
                        Ok(_) => Ok(None),
                        Err(_) => Err(VMError::InvalidItem),
                    }
                    //     }
                    //     Err(_) => Err(VMError::InvalidProof),
                    // }
                },
            )
    }
}

pub struct PhoenixVerify;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for PhoenixVerify {
    const ARGUMENTS: &'static [ValueType] = &[ValueType::I32];
    const RETURN: Option<ValueType> = None;

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let items_ptr = args.get(0)?;

        context
            .top()
            .memory
            .with_direct_access_mut::<Result<Option<RuntimeValue>, VMError>, _>(
                |a| {
                    let items_buf = &a[items_ptr
                        ..items_ptr + (MAX_NOTES_PER_TRANSACTION * ITEM_SIZE)];

                    let items: Result<Vec<TransactionItem>, fermion::Error> =
                        items_buf
                            .chunks(ITEM_SIZE)
                            .map(|bytes| {
                                let item: Item = encoding::decode(bytes)?;
                                Ok(TransactionItem::from(item))
                            })
                            .collect();

                    let mut tx = Transaction::default();

                    for item in items.unwrap() {
                        tx.push(item);
                    }

                    match tx.verify() {
                        Ok(_) => Ok(None),
                        Err(_) => Err(VMError::InvalidItem),
                    }
                },
            )
    }
}
