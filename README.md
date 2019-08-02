# Rust WASM contracts

Work in progreess.

Uses the `wasmi` wasm-interpreter to run contracts.

For heavier loads, such as crypto, the host functions are directly invoked. They can be found in the module `host_fns`

# Tests

To compile and test the contracts, run test.sh