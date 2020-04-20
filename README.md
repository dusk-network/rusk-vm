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

The storage is a trie mapping a key(`Vec<u8>`) to a value (`Vec<u8>`)

For the balance, i chose `u128`, the idea being to move away from Ethereum style 256 bit integers, that are not needed. 128 bit balances should be enough for everyone.

### Contract deployment and Calls

Check the `tests/lib.rs` for an actual usage example of the contract deployment and call interface.

## ABI

The dusk_abi crate is responsible for contract communication with the VM. As well as implementing panic handlers and the boilerplate neccesary to run a contract in a no_std environment.
