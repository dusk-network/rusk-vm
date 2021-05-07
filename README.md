[![Build Status](https://travis-ci.com/dusk-network/rusk-vm.svg?token=h3rscYNqTnqYVQKspVPT&branch=master)](https://travis-ci.com/dusk-network/rusk-vm)

# Rust WASM Virtual Machine

## Usage

To compile and test the contracts, run

```bash
$ make test
```

## Contract deployment and Calls
Check the `tests` for actual usage examples of the contract deployment and call interfaces.

## ABI

The [dusk-abi](https://github.com/dusk-network/dusk-abi) crate is responsible for contract communication with the VM. As well as implementing panic handlers and the boilerplate neccesary to run a contract in a no_std environment.

### Introduction

For more info have a look at the [wiki](https://github.com/dusk-network/rusk-vm/wiki/Introducing)
