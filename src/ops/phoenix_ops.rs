use super::AbiCall;
use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;

use dusk_abi::encoding;
use kelvin::ByteHash;
use phoenix::{
    db, utils, zk, BlsScalar, NoteGenerator, NoteVariant, PublicKey,
    Transaction, TransactionInput, TransactionOutput, TransparentNote,
};
use phoenix_abi::{Note, Nullifier, Proof};
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

pub const DB_PATH: &str = "/tmp/rusk-vm-demo";

const SUCCESS: Result<Option<RuntimeValue>, VMError> =
    Ok(Some(RuntimeValue::I32(1)));

const FAIL: Result<Option<RuntimeValue>, VMError> =
    Ok(Some(RuntimeValue::I32(0)));

pub struct PhoenixStore;

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for PhoenixStore {
    const ARGUMENTS: &'static [ValueType] =
        &[ValueType::I32, ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = Some(ValueType::I32);

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let nullifiers_ptr = args.get(0)?;
        let notes_ptr = args.get(1)?;
        let proof_ptr = args.get(2)?;

        context
            .top()
            .memory
            .with_direct_access_mut::<Result<Option<RuntimeValue>, VMError>, _>(
                |a| {
                    let nullifiers_buf = &a[nullifiers_ptr
                        ..nullifiers_ptr + (Nullifier::MAX * Nullifier::SIZE)];
                    let nullifiers: Result<
                        Vec<TransactionInput>,
                        fermion::Error,
                    > = nullifiers_buf
                        .chunks(Nullifier::SIZE)
                        .map(|bytes| {
                            let nullifier: Nullifier = encoding::decode(bytes)?;
                            let mut item = TransactionInput::default();
                            item.nullifier = nullifier.into();
                            Ok(item)
                        })
                        .collect();
                    let nullifiers = nullifiers.unwrap();

                    let notes_buf =
                        &a[notes_ptr..notes_ptr + (Note::MAX * Note::SIZE)];

                    let notes: Result<Vec<TransactionOutput>, fermion::Error> =
                        notes_buf
                            .chunks(Note::SIZE)
                            .map(|bytes| {
                                let note: Note = encoding::decode(bytes)?;
                                Ok(TransactionOutput::from(note))
                            })
                            .collect();
                    let notes = notes.unwrap();

                    let proof_buf = &a[proof_ptr..proof_ptr + Proof::SIZE];
                    let proof = zk::bytes_to_proof(&proof_buf).unwrap();

                    let mut tx = Transaction::default();

                    nullifiers
                        .iter()
                        .for_each(|nul| tx.push_input(*nul).unwrap());
                    notes
                        .iter()
                        .for_each(|note| tx.push_output(*note).unwrap());

                    tx.set_proof(proof);

                    match db::store(DB_PATH, &tx) {
                        Ok(_) => SUCCESS,
                        Err(_) => FAIL,
                    }
                },
            )
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
        let nullifiers_ptr = args.get(0)?;
        let notes_ptr = args.get(1)?;
        let proof_ptr = args.get(2)?;

        context
            .top()
            .memory
            .with_direct_access_mut::<Result<Option<RuntimeValue>, VMError>, _>(
                |a| {
                    let nullifiers_buf = &a[nullifiers_ptr
                        ..nullifiers_ptr + (Nullifier::MAX * Nullifier::SIZE)];
                    let nullifiers: Result<
                        Vec<TransactionInput>,
                        fermion::Error,
                    > = nullifiers_buf
                        .chunks(Nullifier::SIZE)
                        .map(|bytes| {
                            let nullifier: Nullifier = encoding::decode(bytes)?;
                            let mut item = TransactionInput::default();
                            item.nullifier = nullifier.into();
                            Ok(item)
                        })
                        .collect();
                    let nullifiers = nullifiers.unwrap();

                    let notes_buf =
                        &a[notes_ptr..notes_ptr + (Note::MAX * Note::SIZE)];

                    let notes: Result<Vec<TransactionOutput>, fermion::Error> =
                        notes_buf
                            .chunks(Note::SIZE)
                            .map(|bytes| {
                                let note: Note = encoding::decode(bytes)?;
                                Ok(TransactionOutput::from(note))
                            })
                            .collect();
                    let notes = notes.unwrap();

                    let proof_buf = &a[proof_ptr..proof_ptr + Proof::SIZE];
                    let proof = zk::bytes_to_proof(&proof_buf).unwrap();

                    let mut tx = Transaction::default();

                    nullifiers
                        .iter()
                        .for_each(|nul| tx.push_input(*nul).unwrap());
                    notes
                        .iter()
                        .for_each(|note| tx.push_output(*note).unwrap());

                    tx.set_proof(proof);
                    tx.verify().unwrap();

                    match tx.verify() {
                        Ok(_) => SUCCESS,
                        Err(_) => FAIL,
                    }
                },
            )
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
        let reward = args.get(0)?;
        let pk_ptr = args.get(1)?;

        context
            .top()
            .memory
            .with_direct_access_mut::<Result<Option<RuntimeValue>, VMError>, _>(
                |a| {
                    let pk_bytes = &a[pk_ptr..pk_ptr + 64];
                    let pk = PublicKey::new(
                        utils::deserialize_compressed_jubjub(&pk_bytes[0..32])
                            .unwrap(),
                        utils::deserialize_compressed_jubjub(&pk_bytes[32..64])
                            .unwrap(),
                    );

                    let (output, _) =
                        TransparentNote::output(&pk, reward as u64);

                    let mut tx = Transaction::default();
                    let item = TransactionOutput::new(
                        NoteVariant::Transparent(output),
                        reward as u64,
                        BlsScalar::default(),
                        PublicKey::default(),
                    );
                    tx.push_output(item).unwrap();

                    match db::store(DB_PATH, &tx) {
                        Ok(_) => SUCCESS,
                        Err(_) => FAIL,
                    }
                },
            )
    }
}
