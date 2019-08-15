use std::collections::hash_map::Entry;
use std::collections::HashMap;

use ethereum_types::U256;
use signatory::{ed25519, PublicKeyed};
use signatory_dalek::Ed25519Signer as Signer;

use crate::digest::Digest;
use crate::state::{Account, NetworkState};
use crate::transaction::Transaction;

pub struct ManagedAccount {
    /// The balance in Dusk
    balance: u128,
    /// Nonce to prevent replay-attacks.
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
    pub fn id(&self) -> U256 {
        self.signer
            .public_key()
            .expect("could not get public key from signer (unreachable)")
            .digest()
    }

    pub fn update(&mut self, account: &Account) {
        self.balance = account.balance();
        assert_eq!(self.nonce, account.nonce());
    }

    pub fn balance(&self) -> u128 {
        self.balance
    }

    pub fn send_value(
        &mut self,
        to: U256,
        value: u128,
    ) -> Result<Transaction, ()> {
        if self.balance >= value {
            self.nonce += 1;

            let transaction = Transaction::send_value(
                self.id(),
                to,
                value,
                self.nonce,
                &self.signer,
            );
            Ok(transaction)
        } else {
            Err(())
        }
    }

    pub fn deploy_contract<B: Into<Vec<u8>> + AsRef<[u8]>>(
        &mut self,
        bytecode: B,
        value: u128,
    ) -> Result<(Transaction, U256), ()> {
        if self.balance >= value {
            self.nonce += 1;

            let (transaction, contract_id) = Transaction::deploy_contract(
                self.id(),
                value,
                self.nonce,
                bytecode.into(),
                &self.signer,
            );

            Ok((transaction, contract_id))
        } else {
            Err(())
        }
    }
}

pub struct Wallet(HashMap<String, ManagedAccount>);

impl Wallet {
    pub fn new() -> Self {
        let mut w = Wallet(HashMap::new());
        w.new_account("default").expect("Empty hashmap");
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
        for (_, mut account) in self.0.iter_mut() {
            if let Some(account_state) = state.get_account(&account.id()) {
                account.update(account_state);
            }
        }
    }
}
