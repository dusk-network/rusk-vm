use super::AbiCall;
use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;
use std::convert::{TryFrom, TryInto};
use std::env;
use std::io::Read;
use std::path::Path;

use dusk_abi::encoding;
use kelvin::ByteHash;
use phoenix::{
    db, utils, zk, BlsScalar, Error, NoteGenerator, NoteVariant, PublicKey,
    Transaction, TransactionInput, TransactionOutput, TransparentNote,
};
use phoenix_abi::{Input, Note, Proof};
use wasmi::{RuntimeArgs, RuntimeValue, ValueType};

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
        let inputs_ptr = args.get(0)?;
        let notes_ptr = args.get(1)?;
        let proof_ptr = args.get(2)?;

        context
            .top()
            .memory
            .with_direct_access_mut::<Result<Option<RuntimeValue>, VMError>, _>(
                |a| {
                    let inputs_buf =
                        &a[inputs_ptr..inputs_ptr + (Input::MAX * Input::SIZE)];
                    let inputs: Result<Vec<TransactionInput>, fermion::Error> =
                        inputs_buf
                            .chunks(Input::SIZE)
                            .map(|bytes| {
                                let input: Input = encoding::decode(bytes)?;
                                let item: TransactionInput =
                                    input.try_into().map_err(|_| {
                                        fermion::Error::InvalidRepresentation
                                    })?;
                                Ok(item)
                            })
                            .collect();

                    let notes_buf =
                        &a[notes_ptr..notes_ptr + (Note::MAX * Note::SIZE)];

                    let notes: Result<Vec<TransactionOutput>, fermion::Error> =
                        notes_buf
                            .chunks(Note::SIZE)
                            .map(|bytes| {
                                let note: Note = encoding::decode(bytes)?;
                                Ok(TransactionOutput::try_from(note).map_err(
                                    |_| fermion::Error::InvalidRepresentation,
                                )?)
                            })
                            .collect();

                    let proof_buf = &a[proof_ptr..proof_ptr + Proof::SIZE];
                    let proof = zk::bytes_to_proof(&proof_buf).unwrap();

                    let mut tx = Transaction::default();

                    inputs?
                        .into_iter()
                        .for_each(|input| tx.push_input(input).unwrap());

                    let mut notes = notes.unwrap();

                    let fee_note = notes.pop().unwrap();
                    tx.set_fee(fee_note);

                    notes
                        .into_iter()
                        .for_each(|note| tx.push_output(note).unwrap());

                    tx.set_proof(proof);

                    match db::store(
                        Path::new(&env::var("PHOENIX_DB").unwrap()),
                        &tx,
                    ) {
                        Ok(_) => SUCCESS,
                        Err(e) => FAIL,
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
        let inputs_ptr = args.get(0)?;
        let notes_ptr = args.get(1)?;
        let proof_ptr = args.get(2)?;

        context
            .top()
            .memory
            .with_direct_access_mut::<Result<Option<RuntimeValue>, VMError>, _>(
                |a| {
                    let inputs_buf =
                        &a[inputs_ptr..inputs_ptr + (Input::MAX * Input::SIZE)];
                    let inputs: Result<Vec<TransactionInput>, fermion::Error> =
                        inputs_buf
                            .chunks(Input::SIZE)
                            .map(|bytes| {
                                let input: Input = encoding::decode(bytes)?;
                                let item: TransactionInput =
                                    input.try_into().map_err(|_| {
                                        fermion::Error::InvalidRepresentation
                                    })?;
                                Ok(item)
                            })
                            .collect();

                    let notes_buf =
                        &a[notes_ptr..notes_ptr + (Note::MAX * Note::SIZE)];

                    let mut notes: Result<
                        Vec<TransactionOutput>,
                        fermion::Error,
                    > = notes_buf
                        .chunks(Note::SIZE)
                        .map(|bytes| {
                            let note: Note = encoding::decode(bytes)?;
                            Ok(TransactionOutput::try_from(note).map_err(
                                |_| fermion::Error::InvalidRepresentation,
                            )?)
                        })
                        .collect();

                    let mut proof_buf =
                        &mut a[proof_ptr..proof_ptr + Proof::SIZE];
                    let proof = zk::bytes_to_proof(&proof_buf).unwrap();

                    let mut tx = Transaction::default();

                    inputs?.into_iter().for_each(|input| {
                        if input.nullifier().to_bytes().unwrap() != [0u8; 32] {
                            tx.push_input(input).unwrap()
                        }
                    });

                    let mut notes = notes.unwrap();

                    let fee_note = notes.pop().unwrap();
                    tx.set_fee(fee_note);

                    notes
                        .into_iter()
                        .for_each(|note| tx.push_output(note).unwrap());

                    tx.set_proof(proof);

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

                    match db::store(
                        Path::new(&env::var("PHOENIX_DB").unwrap()),
                        &tx,
                    ) {
                        Ok(_) => SUCCESS,
                        Err(_) => FAIL,
                    }
                },
            )
    }
}
