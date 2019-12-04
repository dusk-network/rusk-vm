# Bringing smart contracts to dusk.network 

## Why?

* Zedger: Account-based Privacy-preserving currency (Dmitry)
* 4 transactions (SPEND, ACCEPT, SETTLE, REDEEM) as building blocks
* our business logic built as smart contracts
* Requirements:
	* blockchain based smart contract platform (i.e., account based)
	* native currency (e.g. DUSK)
	* pairing based ZK-SNARK (e.g. Plonk)

### smart contracts

* a blockchain transition function: `APPLY(S,TX) -> S' or ERROR` (where `S` is the blockchain state and `TX` a transaction).
* smart contract code is executed within a virtual machine (e.g., EVM, Wasm, ...)
* executing the smart contract code is part of the definition of the transition function; every validator executes the smart contract code.


### VM execution
The following pseudo-code (adapted from [here](https://github.com/etclabscore/go-ethereum/blob/development/core/multivm_processor.go#L29-L202)) illustrate a VM execution:


```
// execute the VM
//   vm is a VM instance,
//   bc is our blockchain
Loop:
	for {
		ret := vm.execute_until_blocked_or_done()
		switch ret.typ() {
		
		case RequireNone:
			break Loop // we're done
			
		case RequireAccount:
			address := ret.Address()
			if statedb.Exist(address) {
				vm.CommitAccount(...)
				break
			}
			vm.CommitNonexist(address)
			
		case RequireAccountCode:
			address := ret.Address()
			if statedb.Exist(address) {
				vm.CommitAccountCode(...)
				break
			}
			vm.CommitNonexist(address)
			
		case RequireAccountStorage:
			address := ret.Address()
			...
			if statedb.Exist(address) {
				vm.CommitAccountStorage(...)
				break
			}
			vm.CommitNonexist(address)
			
		case RequireBlockhash:
			number := ret.BlockNumber()
			hash := bc.find(number) // or {}
			vm.CommitBlockhash(number, hash)
		}
	}
// execution done, apply changes to statedb, e.g.: 
//   * increase balance,
//   * decrease balance,
//   * account removed,
//   * account created,
//   * ...

```


## Implementation options
Here are two possible options for bringing smart contracts to dusk.network.

### using dusk-blockchain and dusk-wasm-vm

* [dusk-wasm-vm](https://github.com/dusk-network/dusk-wasm-vm) was developed as a state transition function around an account based blockchain. 
* its state transition function is: `APPLY(S, [TX,..]) -> S' or ERROR`
* TODO1: how does that fit in the current implementation? As I understand it now, [dusk-blockchain](https://github.com/dusk-network/dusk-blockchain/tree/master/pkg/core/chain) primarily deals with UTXOs.
* TODO2: assuming the point above is solved, the following still needs to be done on the existing codebase:
    * sandboxed execution: since smart contracts can be executed by any user on a public network, contracts should only be able to modify their _own_ storage. The sandbox should provide a rollback mechanism, in case the execution fails.
    * deployment mechanism (with binaries stripped from fmt code)
    * ...

### using substrate

* [substrate.dev](https://substrate.dev/) is an open-source framework for building custom blockchain applications. It comes with a [variety of modules (frames)](https://github.com/paritytech/substrate/tree/master/frame) which can be bound together so as to form a (Wasm) runtime. Custom modules can be added to the runtime. Many modules cover functionalities usually needed in blockchain applications (e.g., [election](https://github.com/paritytech/substrate/blob/master/frame/elections/src/lib.rs#L59)).
* substrate is essentially an account based platform. See [here](https://substrate.dev/docs/en/runtime/architecture-of-a-runtime) for an overview of substrate architecture.
* [Ink!](https://github.com/paritytech/ink) belongs to substrate's ecosystems. It is basically a Rust eDSL which allows to quickly develop, test and deploy smart-contracts on substrate. It is conceptually similar to solidity and has the notion of storage (values, hash-maps and linked-maps [ordered]) and events.

AFAIK the substrate framework could be (white-labelled) used to build the following proof of concept, solving all the known use cases:

1. consensus layer (NPoS) 
2. native currency (DUSK)
3. Dmitry transaction model
4. all use cases as smart contracts

The following needs to be done:

  * Plonk integration
      * investigate off-chain workers,
      * see [here](https://github.com/LayerXcom/zero-chain/tree/master/modules/zk-system) for an example
  * implement Dusk protocol as a module
      * Kris started work on the data structures required by Dmitry's paper
  * write smart contracts
      * can start right away against a mock for the Dusk module.

Perhaps also, it may be possible to develop a facade adapting substrate backend to the existing UI.









