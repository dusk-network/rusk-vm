use std::{fmt, io};

use failure::Fail;

mod abi_call;
mod contract;
mod digest;
mod gas;
mod host_fns;
mod state;
mod wallet;

pub use contract::ContractModule;
pub use digest::Digest;
pub use fermion::Error;
pub use gas::{Gas, GasMeter};
pub use host_fns::StandardABI;
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
