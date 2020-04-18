[![Build Status](https://travis-ci.com/dusk-network/rusk-vm.svg?token=h3rscYNqTnqYVQKspVPT&branch=master)](https://travis-ci.com/dusk-network/rusk-vm)

# Rust WASM Virtual Machine

## Usage

To compile and test the contracts, run

```bash
$ make test
```

## Design

The design idea of the VM is _everything is a contract_. There are no separation between "accounts" and contracts, accounts are simply contracts programmed to behave like accounts.

The state is a trie mapping `32-byte Hash` to a tuple containing `(balance: u128, bytecode: Vec<u8>, Storage)`

The storage is a trie mapping `32-byte key` to Vec<u8> of `STORAGE_VAL_SIZE`

For the balance, i chose `u128`, the idea being to move away from Ethereum style 256 bit integers, that are not needed. 128 bit balances should be enough for everyone.

### Constants (to be discussed)

```rust
/// The maximum size of contract call arguments and return values
pub const CALL_DATA_SIZE: usize = 1024 * 16;
/// The maximum size of values in contract storage
pub const STORAGE_VALUE_SIZE: usize = 1024 * 4;
/// The size of keys for contract storage
pub const STORAGE_KEY_SIZE: usize = 32;
/// The maximum length of contract panic messages
pub const PANIC_BUFFER_SIZE: usize = 1024 * 16;
/// The maximum length of contract debug messages
pub const DEBUG_BUFFER_SIZE: usize = 1024 * 16;
```

### Interaction

Interfacing with contract calls is done through the type `dusk_abi::ContractCall<R>` where R encodes the expected return value.

Check the `tests/lib.rs` for an actual usage example of this interface.

## ABI

The dusk_abi crate is responsible for contract communication with the VM. As well as implementing panic handlers and the boilerplate neccesary to run a contract in a no_std environment.
