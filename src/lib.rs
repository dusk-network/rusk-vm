use std::collections::HashMap;
use std::io::Write;

use blake2_rfc::blake2b::Blake2b;
use signatory::{
    ed25519::{self, PublicKey},
    PublicKeyed,
};
use signatory_dalek::Ed25519Signer;
use wasmi::{ImportsBuilder, Module, ModuleInstance, RuntimeValue};

mod host_fns;

use host_fns::HostExternals;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Digest([u8; 32]);

struct Account {
    #[allow(unused)]
    balance: u128,
}

impl Digest {
    fn new(bytes: &[u8]) -> Self {
        let mut digest = [0u8; 32];
        let mut state = Blake2b::new(32);
        state.update(bytes);
        digest
            .as_mut()
            .write(state.finalize().as_bytes())
            .expect("in-memory write");
        Digest(digest)
    }
}

#[derive(Default)]
pub struct NetworkState {
    contracts: HashMap<Digest, Module>,
    accounts: HashMap<PublicKey, Account>,
}

impl NetworkState {
    pub fn new_account(&mut self, balance: u128) -> PublicKey {
        let seed = ed25519::Seed::generate();
        let signer = Ed25519Signer::from(&seed);
        let pk = signer.public_key().unwrap();

        self.accounts.insert(pk.clone(), Account { balance });
        pk
    }

    pub fn new_contract(&mut self, bytecode: &[u8]) -> Digest {
        let module = wasmi::Module::from_buffer(bytecode)
            .expect("failed to parse bytecode");

        let hash = Digest::new(bytecode);

        self.contracts.insert(hash.clone(), module);
        hash
    }

    pub fn call(
        &mut self,
        contract_hash: &Digest,
        method: &str,
        args: &[RuntimeValue],
    ) {
        if let Some(contract) = self.contracts.get(&contract_hash) {
            let imports =
                ImportsBuilder::new().with_resolver("env", &HostExternals);

            let instance = ModuleInstance::new(&contract, &imports)
                .expect("failed to instantiate wasm module")
                .assert_no_start();

            let result =
                instance.invoke_export(method, args, &mut HostExternals);

            println!("{:?}", result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut network = NetworkState::default();
        let _alice_pk = network.new_account(1000);
        let _bob_pk = network.new_account(0);

        let contract = network.new_contract(include_bytes!(
            "../test_contracts/basic/target/wasm32-unknown-unknown/release/test_contract.wasm"
        ));

        // network.call(&contract, "saturating_sub", &[RuntimeValue::I32(5)]);
        network.call(&contract, "trampoline", &[]);
    }
}
