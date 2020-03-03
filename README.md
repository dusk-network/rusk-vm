[![Build Status](https://travis-ci.com/dusk-network/rusk-vm.svg?branch=master)](https://travis-ci.com/dusk-network/rusk-vm)

# Rust WASM contracts

## Usage

To compile and test the contracts, run

```bash
$ ./test.sh default_account
$ ./test.sh factorial
```

## Design

The design idea of the VM is _everything is a contract_. There are no separation between "accounts" and contracts, accounts are simply contracts programmed to behave like accounts.

The state a trie mapping `32-byte Hash` to a tuple containing `(balance: u128, bytecode: Vec<u8>, Storage)`

The storage is a trie mapping `32-byte key` to Vec<u8> of `STORAGE_VAL_SIZE`

For the balance, i chose `u128`, the idea being to move away from Ethereum style 256 bit integers, that are not needed. 128 bit balances should be enough for everyone.

### Constants (to be discussed)

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

### Interaction

There is also no difference between a `call` and a `transaction` from the VM standpoint. Every call carries a value, in dusk, to be transfered along with the call. A transfer is simply a call with no data (data of zero length).

The entry point of the call is responsible for paying for the transaction, in most cases this will be an Account or UTXO contract. If the transaction does not trigger a payment of gas/dusk in the first step, it is regarded as invalid.

### Default account contract

The default account acts as a Ethereum account, the full logic of the contract (as of 2019-10-09) is provided here:

```rust
#![no_std]
use dusk_abi::{self, encoding, ContractCall, Signature, CALL_DATA_SIZE, H256};
use serde::{Deserialize, Serialize};

const NONCE: [u8; 1] = [0u8];

#[no_mangle]
static PUBLIC_KEY: [u8; 32] = [0u8; 32];

#[derive(Serialize, Deserialize, Debug)]
pub enum AccountCall<'a> {
    CallThrough {
        to: H256,
        amount: u128,
        call_data: &'a [u8],
        nonce: u64,
        signature: Signature,
    },
    Balance,
}

#[no_mangle]
pub fn call() {
    let mut buffer = [0u8; CALL_DATA_SIZE];
    let data: AccountCall = dusk_abi::call_data(&mut buffer);
    match data {
        // Transfer funds and call through to another contract
        AccountCall::CallThrough {
            to,
            amount,
            nonce,
            signature,
            call_data,
            ..
        } => {
            let current_nonce = dusk_abi::get_storage(&NONCE).unwrap();

            assert!(nonce == current_nonce);

            let mut verify_buf = [0u8; 32 + 16 + 8];
            let encoded =
                encoding::encode(&(to, amount, nonce), &mut verify_buf)
                    .expect("buffer insufficient");

            if dusk_abi::verify_ed25519_signature(
                &PUBLIC_KEY,
                &signature,
                encoded,
            ) {
                let mut call = ContractCall::<()>::new_raw(call_data);
                dusk_abi::call_contract(&to, amount, &call);
                dusk_abi::set_storage(&NONCE, current_nonce + 1);
            } else {
                panic!("invalid signature!");
            }
        }
        // Return the account balance
        AccountCall::Balance => {
            let balance = dusk_abi::balance();
            dusk_abi::ret(balance);
        }
    }
}

#[no_mangle]
pub fn deploy() {
    // Set the initial nonce to zero
    dusk_abi::set_storage(&NONCE, 0u64)
}
```

The contract itself is responsible for checking signatures and nonces of the calls, and optionally passing on the call to another contract.

Interfacing with contract calls is done throug the type `dusk_abi::ContractCall<R>` where R encodes the expected return value.

In `src/interfaces/default_contract.rs` the top-level interface for the default account is specified as such:

```rust
pub struct DefaultAccount;

impl DefaultAccount {
    pub fn transfer(
        signer: &Signer,
        to: H256,
        amount: u128,
        nonce: u64,
    ) -> ContractCall<()> {
        let mut buf = [0u8; 32 + 16 + 8];
        let encoded = encoding::encode(&(to, amount, nonce), &mut buf)
            .expect("static buffer too small");
        let signature = signer.sign(encoded);

        let signature = Signature::from_slice(signature.as_slice());

        ContractCall::new(AccountCall::CallThrough {
            to,
            amount,
            nonce,
            call_data: &[],
            signature,
        })
        .expect("CALL_DATA too small")
    }

    pub fn balance() -> ContractCall<u128> {
        ContractCall::new(AccountCall::Balance).expect("CALL_DATA too small")
    }
}
```

Check the `src/lib.rs` for an actual usage example of this interface.

## ABI

The dusk_abi crate is responsible for contract communication with the VM. As well as implementing panic handlers and the boilerplate neccesary to run a contract in a no_std environment.

## WASM

At the moment, the wasm binaries produced by linking `dusk_abi` are relatively large. Most of this has to do with error formatting and debug code being linked in. In the near future this will be hidden behind a `debug` feature. Allowing for easy debugging while development and small binaries when deploying.
