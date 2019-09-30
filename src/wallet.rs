use std::collections::hash_map::Entry;
use std::collections::HashMap;

use dusk_abi::types::H256;
use failure::{bail, Error};
use signatory::{ed25519, PublicKeyed};
use signatory_dalek::Ed25519Signer as Signer;

use crate::digest::Digest;
use crate::state::{Contract, NetworkState};
// use crate::transaction::Transaction;

const SECRET: [u8; 32] = *b"super secret key is super secret";

pub struct ManagedAccount {
    /// The balance in Dusk
    balance: u128,
    nonce: u128,
    signer: Signer,
}

impl Default for ManagedAccount {
    fn default() -> Self {
        let seed = ed25519::Seed::generate();
        let signer = Signer::from(&seed);

        ManagedAccount {
            balance: 0,
            nonce: 0,
            signer,
        }
    }
}

impl ManagedAccount {
    pub fn id(&self) -> H256 {
        self.signer
            .public_key()
            .expect("could not get public key from signer (unreachable)")
            .digest()
    }

    pub fn from_seed(seed: [u8; 32]) -> Self {
        let seed = ed25519::Seed::new(seed);
        let signer = Signer::from(&seed);

        ManagedAccount {
            balance: 0,
            nonce: 0,
            signer,
        }
    }

    pub fn update(&mut self, contract: &Contract) {
        self.balance = contract.balance();
    }

    pub fn balance(&self) -> u128 {
        self.balance
    }

    pub fn public_key(&self) -> signatory::ed25519::PublicKey {
        self.signer.public_key().expect("never fails")
    }

    // pub fn call_contract(
    //     &mut self,
    //     contract_id: &H256,
    //     value: u128,
    //     data: &[u8],
    // ) -> Result<Transaction, Error> {
    //     if self.balance >= value {
    //         self.nonce += 1;

    //         let transaction = Transaction::call_contract(
    //             self.id(),
    //             contract_id.clone(),
    //             self.nonce,
    //             value,
    //             data.into(),
    //             &self.signer,
    //         );
    //         Ok(transaction)
    //     } else {
    //         bail!("Insufficient balance")
    //     }
    // }

    // pub fn deploy_contract<B: Into<Vec<u8>> + AsRef<[u8]>>(
    //     &mut self,
    //     bytecode: B,
    //     value: u128,
    // ) -> Result<(Transaction, H256), ()> {
    //     if self.balance >= value {
    //         self.nonce += 1;

    //         let (transaction, contract_id) = Transaction::deploy_contract(
    //             self.id(),
    //             value,
    //             self.nonce,
    //             bytecode.into(),
    //             &self.signer,
    //         );

    //         Ok((transaction, contract_id))
    //     } else {
    //         Err(())
    //     }
    // }
}

pub struct Wallet(HashMap<String, ManagedAccount>);

impl Wallet {
    pub fn new() -> Self {
        let mut w = Wallet(HashMap::new());
        w.new_account("default").expect("conflict in empty hashmap");
        w
    }

    pub(crate) fn genesis() -> Self {
        let mut w = Wallet(HashMap::new());
        w.new_account_with_seed("default", SECRET)
            .expect("conflict in empty hashmap");
        w
    }

    pub fn default_account(&self) -> &ManagedAccount {
        self.0.get("default").expect("No default account")
    }

    pub fn default_account_mut(&mut self) -> &mut ManagedAccount {
        self.0.get_mut("default").expect("No default account")
    }

    /// Create a new account with the given name,
    /// Returns an error if an account with that name already exists
    pub fn new_account<S: Into<String>>(
        &mut self,
        name: S,
    ) -> Result<&mut ManagedAccount, ()> {
        match self.0.entry(name.into()) {
            Entry::Vacant(v) => Ok(v.insert(ManagedAccount::default())),
            _ => Err(()),
        }
    }

    /// Create a new account with the given name and given seed,
    /// Returns an error if an account with that name already exists
    pub fn new_account_with_seed<S: Into<String>>(
        &mut self,
        name: S,
        seed: [u8; 32],
    ) -> Result<&mut ManagedAccount, ()> {
        match self.0.entry(name.into()) {
            Entry::Vacant(v) => Ok(v.insert(ManagedAccount::from_seed(seed))),
            _ => Err(()),
        }
    }

    pub fn get_account(&self, name: &str) -> Option<&ManagedAccount> {
        self.0.get(name)
    }

    pub fn get_account_mut(
        &mut self,
        name: &str,
    ) -> Option<&mut ManagedAccount> {
        self.0.get_mut(name)
    }

    pub fn sync(&mut self, state: &NetworkState) {
        for (_, contract) in self.0.iter_mut() {
            if let Some(account_state) = state.get_contract(&contract.id()) {
                contract.update(account_state);
            }
        }
    }
}
