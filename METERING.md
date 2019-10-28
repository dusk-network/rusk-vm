# Wasm and Metering

## Parity Substrate 

[Parity Substrate](https://www.parity.io/substrate/) is a framework for building blockchains, created by Gavin Wood ([Ethereum](https://ethereum.github.io/yellowpaper/paper.pdf).) 

Parity Substrates uses Wasm as a VM for its smart-contracts, [with the following restrictions](https://github.com/paritytech/substrate/blob/master/srml/contracts/src/wasm/prepare.rs#L341-L344):

* be a valid Wasm module,
* which doesn't declare internal linear memories (can only import memories from the environment), and
* has only one table, *not too big* (configurable);
* does not use floating point types (`F32`|`F64`), and
* exports the `call` and `deploy` entry-points.

Upon passing these verifications, the module is instrumented as following:

* gas counters are injected (described [here](https://github.com/paritytech/wasm-utils/blob/master/src/gas/mod.rs#L389-L422)),
* add [deterministic stack limiting](https://wiki.parity.io/WebAssembly-StackHeight) instrumentation. 

The cost of various wasm/vm parameters is [configurable](https://github.com/paritytech/substrate/blob/master/srml/contracts/src/lib.rs#L893-L954). Both metering utilities (gas/stack-height) are available in the [pwasm_util](https://crates.io/crates/pwasm-utils) Rust crate.

## Dusk.network

* can any of the parity code be of use for the Dusk.network VM?
* what are our requirements in term of performance?

