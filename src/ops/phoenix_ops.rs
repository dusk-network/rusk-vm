use super::AbiCall;
use crate::call_context::{ArgsExt, CallContext, Resolver};
use crate::VMError;
use dusk_abi::PodExt;
use kelvin::ByteHash;
use phoenix::{
    db, utils, zk, BlsScalar, NoteGenerator, NoteVariant, PublicKey,
    Transaction, TransactionInput, TransactionOutput, TransparentNote,
};
use phoenix_abi::{Input, Note, Proof};
use std::convert::{TryFrom, TryInto};
use std::env;
use std::path::Path;
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

fn get_tx<S, H>(
    context: &mut CallContext<S, H>,
    args: RuntimeArgs,
) -> Result<Transaction, VMError>
where
    S: Resolver<H>,
    H: ByteHash,
{
    let inputs_ptr = args.get(0)? as usize;
    let notes_ptr = args.get(1)? as usize;
    let proof_ptr = args.get(2)? as usize;

    let (inputs_buf, notes_buf, proof_buf): (Vec<_>, Vec<_>, Vec<_>) = context
        .memory(|m| {
            (
                m[inputs_ptr..inputs_ptr + (Input::MAX * Input::SIZE)].into(),
                m[notes_ptr..notes_ptr + (Note::MAX * Note::SIZE)].into(),
                m[proof_ptr..proof_ptr + Proof::SIZE].into(),
            )
        });

    let inputs: Vec<TransactionInput> = inputs_buf
        .chunks(Input::SIZE)
        .take_while(|bytes| has_value(bytes))
        .map(|bytes| {
            let input = Input::from_slice(bytes);
            let item: TransactionInput = input
                .try_into()
                .unwrap_or_else(|_| panic!("invalid representation"));
            item
        })
        .collect();

    let mut notes: Vec<TransactionOutput> = notes_buf
        .chunks(Note::SIZE)
        .take_while(|bytes| has_value(bytes))
        .map(|bytes| {
            let note = Note::from_slice(bytes);
            TransactionOutput::try_from(note)
                .unwrap_or_else(|_| panic!("invalid representation"))
        })
        .collect();

    let proof = zk::bytes_to_proof(&proof_buf).unwrap();

    let mut tx = Transaction::default();

    inputs
        .into_iter()
        .for_each(|input| tx.push_input(input).unwrap());
    let fee_note = notes.pop().unwrap();
    tx.set_fee(fee_note);

    notes
        .into_iter()
        .for_each(|note| tx.push_output(note).unwrap());

    tx.set_proof(proof);

    Ok(tx)
}

impl<S: Resolver<H>, H: ByteHash> AbiCall<S, H> for PhoenixStore {
    const ARGUMENTS: &'static [ValueType] =
        &[ValueType::I32, ValueType::I32, ValueType::I32];
    const RETURN: Option<ValueType> = Some(ValueType::I32);

    fn call(
        context: &mut CallContext<S, H>,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, VMError> {
        let tx = get_tx(context, args)?;
        match db::store(Path::new(&env::var("PHOENIX_DB").unwrap()), &tx) {
            Ok(_) => SUCCESS,
            Err(_) => FAIL,
        }
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
        let mut tx = get_tx(context, args)?;

        match tx.verify() {
            Ok(_) => SUCCESS,
            Err(_) => FAIL,
        }
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
        let reward_ptr = args.get(0)? as usize;
        let pk_ptr = args.get(1)? as usize;

        let (reward, pk_buf): (u64, Vec<_>) = context.memory(|m| {
            (
                u64::from_slice(&m[reward_ptr..]),
                m[pk_ptr..pk_ptr + 64].into(),
            )
        });

        let pk = PublicKey::new(
            utils::deserialize_compressed_jubjub(&pk_buf[0..32]).unwrap(),
            utils::deserialize_compressed_jubjub(&pk_buf[32..64]).unwrap(),
        );

        let (output, _) = TransparentNote::output(&pk, reward);

        let mut tx = Transaction::default();
        let item = TransactionOutput::new(
            NoteVariant::Transparent(output),
            reward,
            BlsScalar::default(),
            PublicKey::default(),
        );
        tx.push_output(item).unwrap();

        match db::store(Path::new(&env::var("PHOENIX_DB").unwrap()), &tx) {
            Ok(_) => SUCCESS,
            Err(_) => FAIL,
        }
    }
}
