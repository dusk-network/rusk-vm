use std::{fmt, io};

use failure::Fail;

mod contract;
mod digest;
mod gas;
mod helpers;
mod host_fns;
mod interfaces;
mod state;
mod wallet;

pub use contract::ContractModule;
pub use fermion::Error;
pub use gas::Gas;
pub use interfaces::DefaultAccount;
pub use state::NetworkState;
pub use wallet::Wallet;

#[derive(Debug, Fail)]
pub enum VMError {
    MissingArgument,
    ContractPanic(String),
    MemoryNotFound,
    InvalidApiCall,
    InvalidUtf8,
    InvalidEd25519PublicKey,
    InvalidEd25519Signature,
    ContractReturn,
    OutOfGas,
    UnknownContract,
    WASMError(failure::Error),
    Trap(wasmi::Trap),
    IOError(io::Error),
    WasmiError(wasmi::Error),
    SerializationError(fermion::Error),
}

impl From<io::Error> for VMError {
    fn from(e: io::Error) -> Self {
        VMError::IOError(e)
    }
}

impl From<fermion::Error> for VMError {
    fn from(e: fermion::Error) -> Self {
        VMError::SerializationError(e)
    }
}

impl From<wasmi::Error> for VMError {
    fn from(e: wasmi::Error) -> Self {
        VMError::WasmiError(e)
    }
}

impl From<wasmi::Trap> for VMError {
    fn from(e: wasmi::Trap) -> Self {
        VMError::Trap(e)
    }
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
            VMError::OutOfGas => write!(f, "Out of Gas Error")?,
            VMError::WASMError(e) => write!(f, "WASM Error ({:?})", e)?,
            VMError::MemoryNotFound => write!(f, "Memory not found")?,
            VMError::SerializationError(e) => {
                write!(f, "Serialization Error ({:?})", e)?
            }
            VMError::InvalidApiCall => write!(f, "Invalid Api Call")?,
            VMError::IOError(e) => write!(f, "Input/Output Error ({:?})", e)?,
            VMError::Trap(_) => unreachable!(),
            VMError::WasmiError(e) => write!(f, "WASMI Error ({:?})", e)?,
            VMError::UnknownContract => write!(f, "Unknown Contract")?,
        }
        Ok(())
    }
}

/// Definition of the cost schedule and other parameterizations for wasm vm.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq)]
pub struct Schedule {
    /// Version of the schedule.
    pub version: u32,

    /// Cost of putting a byte of code into storage.
    pub put_code_per_byte_cost: Gas,

    /// Gas cost of a growing memory by single page.
    pub grow_mem_cost: Gas,

    /// Gas cost of a regular operation.
    pub regular_op_cost: Gas,

    /// Gas cost per one byte returned.
    pub return_data_per_byte_cost: Gas,

    /// Gas cost to deposit an event; the per-byte portion.
    pub event_data_per_byte_cost: Gas,

    /// Gas cost to deposit an event; the cost per topic.
    pub event_per_topic_cost: Gas,

    /// Gas cost to deposit an event; the base.
    pub event_base_cost: Gas,

    /// Base gas cost to call into a contract.
    pub call_base_cost: Gas,

    /// Base gas cost to instantiate a contract.
    pub instantiate_base_cost: Gas,

    /// Gas cost per one byte read from the sandbox memory.
    pub sandbox_data_read_cost: Gas,

    /// Gas cost per one byte written to the sandbox memory.
    pub sandbox_data_write_cost: Gas,

    /// The maximum number of topics supported by an event.
    pub max_event_topics: u32,

    /// Maximum allowed stack height.
    ///
    /// See https://wiki.parity.io/WebAssembly-StackHeight to find out
    /// how the stack frame cost is calculated.
    pub max_stack_height: u32,

    /// Maximum number of memory pages allowed for a contract.
    pub max_memory_pages: u32,

    /// Maximum allowed size of a declared table.
    pub max_table_size: u32,

    /// Whether the `ext_println` function is allowed to be used contracts.
    /// MUST only be enabled for `dev` chains, NOT for production chains
    pub enable_println: bool,

    /// The maximum length of a subject used for PRNG generation.
    pub max_subject_len: u32,
}

impl Default for Schedule {
    fn default() -> Schedule {
        Schedule {
            version: 0,
            put_code_per_byte_cost: 1,
            grow_mem_cost: 1,
            regular_op_cost: 1,
            return_data_per_byte_cost: 1,
            event_data_per_byte_cost: 1,
            event_per_topic_cost: 1,
            event_base_cost: 1,
            call_base_cost: 135,
            instantiate_base_cost: 175,
            sandbox_data_read_cost: 1,
            sandbox_data_write_cost: 1,
            max_event_topics: 4,
            max_stack_height: 64 * 1024,
            max_memory_pages: 16,
            max_table_size: 16 * 1024,
            enable_println: false,
            max_subject_len: 32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use digest::Digest;

    #[test]
    fn default_account() {
        let mut wallet = Wallet::new();
        let schedule = Schedule::default();
        let mut genesis_builder =
            ContractModule::new(contract_code!("default_account"), &schedule)
                .unwrap();

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
        let schedule = Schedule::default();
        let mut account_builder =
            ContractModule::new(contract_code!("default_account"), &schedule)
                .unwrap();

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

        let schedule = Schedule::default();

        let genesis_builder =
            ContractModule::new(contract_code!("add"), &schedule).unwrap();

        let genesis = genesis_builder.build().unwrap();

        // New genesis network with initial value
        let mut network =
            NetworkState::genesis(genesis, 1_000_000_000).unwrap();

        let genesis_id = *network.genesis_id();

        let mut gas_meter = gas::GasMeter::with_limit(393_588);
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
        let schedule = Schedule::default();
        let genesis_builder =
            ContractModule::new(contract_code!("factorial"), &schedule)
                .unwrap();

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
        let schedule = Schedule::default();
        let genesis_builder =
            ContractModule::new(contract_code!("factorial"), &schedule)
                .unwrap();

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

        let schedule = Schedule::default();
        let genesis_builder =
            ContractModule::new(contract_code!("panic"), &schedule).unwrap();

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
