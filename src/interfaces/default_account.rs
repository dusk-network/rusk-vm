use default_account::AccountCall;
use dusk_abi::{encoding, ContractCall, Signature, H256};
use signatory::signature::{Signature as _, Signer as _};
use signatory_dalek::Ed25519Signer as Signer;

pub struct DefaultAccount;

impl DefaultAccount {
    pub fn transfer(
        signer: &Signer,
        to: H256,
        amount: u128,
        nonce: u64,
    ) -> ContractCall<()> {
        let mut buf = [0u8; 32 + 16 + 8];
        let encoded = encoding::encode(&(to, amount, nonce), &mut buf)
            .expect("static buffer too small");
        let signature = signer.sign(encoded);

        let signature = Signature::from_slice(signature.as_slice());

        ContractCall::new(AccountCall::CallThrough {
            to,
            amount,
            nonce,
            call_data: &[],
            signature,
        })
        .expect("CALL_DATA too small")
    }

    pub fn balance() -> ContractCall<u128> {
        ContractCall::new(AccountCall::Balance).expect("CALL_DATA too small")
    }
}
