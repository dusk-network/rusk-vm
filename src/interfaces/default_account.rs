#[rustfmt::skip]
use ::default_account::AccountCall;
use dusk_abi::{encoding, Error, Signature, H256};
use signatory::{Signature as _, Signer as _};
use signatory_dalek::Ed25519Signer as Signer;

pub struct DefaultAccount;

impl DefaultAccount {
    pub fn transfer(
        signer: &Signer,
        to: H256,
        amount: u128,
        nonce: u64,
    ) -> Result<AccountCall<'static>, Error> {
        let mut buf = [0u8; 32 + 16 + 8];
        let encoded = encoding::encode(&(to, amount, nonce), &mut buf)?;
        let signature = signer.sign(encoded);

        let signature = Signature::from_slice(signature.as_slice());

        Ok(AccountCall::new(to, amount, &[], nonce, signature))
    }
}
