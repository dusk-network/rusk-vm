use std::fmt;

use failure::Fail;

mod contract;
mod digest;
mod gas;
mod helpers;
mod host_fns;
mod interfaces;
mod state;
mod wallet;

pub use contract::ContractBuilder;
pub use interfaces::DefaultAccount;
pub use state::NetworkState;
pub use wallet::Wallet;

#[derive(Debug, Fail)]
pub enum VMError {
    MissingArgument,
    ContractPanic(String),
    InvalidUtf8,
    InvalidEd25519PublicKey,
    InvalidEd25519Signature,
    ContractReturn,
    SerializationError,
    OutOfGas,
    WASMError(failure::Error),
}

impl wasmi::HostError for VMError {}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VMError::MissingArgument => write!(f, "Missing Argument")?,
            VMError::ContractPanic(string) => {
                write!(f, "Contract panic \"{}\"", string)?
            }
            VMError::InvalidUtf8 => write!(f, "Invalid UTF-8")?,
            VMError::InvalidEd25519PublicKey => {
                write!(f, "Invalid Ed25519 Public Key")?
            }
            VMError::InvalidEd25519Signature => {
                write!(f, "Invalid Ed25519 Signature")?
            }
            VMError::ContractReturn => write!(f, "Contract Return")?,
            VMError::SerializationError => write!(f, "Serialization Error")?,
            VMError::OutOfGas => write!(f, "Out of Gas Error")?,
            VMError::WASMError(e) => write!(f, "WASM Error ({:?})", e)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use digest::Digest;

    #[test]
    fn default_account() {
        let mut wallet = Wallet::new();

        let mut genesis_builder =
            ContractBuilder::new(contract_code!("default_account")).unwrap();

        let pub_key = wallet.default_account().public_key();
        genesis_builder
            .set_parameter("PUBLIC_KEY", pub_key)
            .unwrap();

        let genesis = genesis_builder.build().unwrap();

        // New genesis network with initial value
        let mut network =
            NetworkState::genesis(genesis, 1_000_000_000).unwrap();

        let genesis_id = *network.genesis_id();

        // check balance of genesis account
        assert_eq!(
            network
                .call_contract(genesis_id, DefaultAccount::balance())
                .unwrap(),
            1_000_000_000
        );

        // setup a secondary account

        wallet.new_account("alice").unwrap();

        let mut account_builder =
            ContractBuilder::new(contract_code!("default_account")).unwrap();

        let alice_pub_key = wallet.get_account("alice").unwrap().public_key();
        account_builder
            .set_parameter("PUBLIC_KEY", alice_pub_key)
            .unwrap();

        let alice_account = account_builder.build().unwrap();
        let alice_account_id = alice_account.digest();
        // transfer 1000 to alice from genesis account

        let genesis_signer = wallet.default_account().signer();

        let call = DefaultAccount::transfer(
            genesis_signer,
            alice_account.digest(),
            1000,
            0,
        );

        network.call_contract(genesis_id, call).unwrap();

        // deploy/reveal alices contract

        network.deploy_contract(alice_account).unwrap();

        // check balances

        assert_eq!(
            network
                .call_contract(alice_account_id, DefaultAccount::balance())
                .unwrap(),
            1_000,
        );

        assert_eq!(
            network
                .call_contract(genesis_id, DefaultAccount::balance())
                .unwrap(),
            1_000_000_000 - 1_000
        );
    }

    #[test]
    fn add() {
        use add::add;

        let genesis_builder =
            ContractBuilder::new(contract_code!("add")).unwrap();

        let genesis = genesis_builder.build().unwrap();

        // New genesis network with initial value
        let mut network =
            NetworkState::genesis(genesis, 1_000_000_000).unwrap();

        let genesis_id = *network.genesis_id();

        let mut gas_meter = gas::GasMeter::with_limit(1_000);
        println!(
            "Before call: gas_meter={:?} (spent={})",
            gas_meter,
            gas_meter.spent()
        );

        let (a, b) = (12, 40);
        assert_eq!(
            network
                .call_contract_with_limit(genesis_id, add(a, b), &mut gas_meter)
                .unwrap(),
            a + b
        );
        println!(
            "After call: gas_meter={:?} (spent={})",
            gas_meter,
            gas_meter.spent()
        );
    }

    #[test]
    fn factorial() {
        use factorial::factorial;

        fn factorial_reference(n: u64) -> u64 {
            if n <= 1 {
                1
            } else {
                n * factorial_reference(n - 1)
            }
        }

        let genesis_builder =
            ContractBuilder::new(contract_code!("factorial")).unwrap();

        let genesis = genesis_builder.build().unwrap();

        // New genesis network with initial value
        let mut network =
            NetworkState::genesis(genesis, 1_000_000_000).unwrap();

        let genesis_id = *network.genesis_id();

        let n = 6;
        assert_eq!(
            network.call_contract(genesis_id, factorial(n)).unwrap(),
            factorial_reference(n)
        );
    }

    #[test]
    fn factorial_with_limit() {
        use factorial::factorial;

        fn factorial_reference(n: u64) -> u64 {
            if n <= 1 {
                1
            } else {
                n * factorial_reference(n - 1)
            }
        }

        let genesis_builder =
            ContractBuilder::new(contract_code!("factorial")).unwrap();

        let genesis = genesis_builder.build().unwrap();

        // New genesis network with initial value
        let mut network =
            NetworkState::genesis(genesis, 1_000_000_000).unwrap();

        let genesis_id = *network.genesis_id();
        let mut gas_meter = gas::GasMeter::with_limit(1_000_000_000);
        println!(
            "Before call: gas_meter={:?} (spent={})",
            gas_meter,
            gas_meter.spent()
        );

        let n = 6;
        assert_eq!(
            network
                .call_contract_with_limit(
                    genesis_id,
                    factorial(n),
                    &mut gas_meter
                )
                .unwrap(),
            factorial_reference(n)
        );

        println!(
            "After call: gas_meter={:?} (spent={})",
            gas_meter,
            gas_meter.spent()
        );
    }

    #[test]
    #[should_panic]
    fn panic_propagation() {
        use dusk_abi::ContractCall;

        let genesis_builder =
            ContractBuilder::new(contract_code!("panic")).unwrap();

        let genesis = genesis_builder.build().unwrap();

        // New genesis network with initial value
        let mut network =
            NetworkState::genesis(genesis, 1_000_000_000).unwrap();

        let genesis_id = *network.genesis_id();

        network
            .call_contract::<()>(genesis_id, ContractCall::nil())
            .unwrap();
    }
}
