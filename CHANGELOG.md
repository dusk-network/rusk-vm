# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added

- Introduce lang items as in `dusk-abi` [#374]
- Add `HostCosts` config structure [#205]
- Add `NetworkState::with_config` for instantiating a VM with the given configuration [#304]
- Add `NetworkState::config` for getting instance configuration [#304]
- Add `Config` and `OpCosts` structs [#304]
- Enable reading the state root [#265]
- Add `staged` state in addition to the existing `head` and `origin` [#302]
- Add `unstage` method to remove changes from `staged` [#302]
- Add `push` method to push the committed changes to `origin` [#302]
- Add `exhaust` method to `GasMeter` [#308]
- Add a private `update` method to `GasMeter` [#308]
- Add `Instance` field to `StackFrame` [#308]
- Add `CallContext::top_mut()` [#308]
- Add new `CallContext::gas_meter` method in `ops` module

### Changed

- Change host functions to charge their cost according to config [#205]
- Change `Config` to include `host_costs` [#205]
- Change persistence to include configuration hash [#304]
- Refactor configuration to be a static `Config` per instance [#304]
- Place both `Contract::state` and `Contract::code` behind `microkelvin::Link` [#333]
- Change `commit` method to commit the changes from `staged` to `head` [#302]
- Change `register_host_module` to be an associated function
- Replace `GasMeter::set_left(0)` with `GasMeter::exhaust()` [#308]
- Change `CallContext::gas_meter()` to update the gas meter before return it [#308]

### Removed

- Remove `NetworkState::with_schedule` [#304]
- Remove `ModuleConfig` and `NetworkState::get_module_config` [#304]
- Remove `Schedule` as configuration structure [#304]
- Remove `set_left` method from `GasMeter` [#308]
- Remove legacy `gas` host function
- Remove `CallContext::gas_meter_mut()` [#308]
- Remove `Gas` host function implementation from `ops` module

## [0.9.0] - 2022-02-02

### Added

- Add `NetworkStateId` structure to handle both origin/head `PersistedId`
- Add utility methods to `NetworkStateId` to save / load from file
- Add `persist` module under `state`

### Changed

- Update rust toolchain
- Change `NetworkState::persist` to store both origin/head
- Change `NetworkState::persist()` to accept the same type of `Persistence::persist()`
- Change `NetworkState::restore` to restore both origin/head
- Update integration tests
- Change the rust toolchain

### Removed

- Remove host modules from Rusk-VM's instances

### Fixed

- Fix minor nits on tests

## [0.7.0] - 2022-01-19

### Added

- Add capability of reading the state root [#265]
- Add log tracing for easier debugging [#83]
- Add support for transaction rollbacks [#263]
- Add cache for `wasmer` modules [#251]
- Add `wasmer` instrumentation [#247]
- Add CHANGELOG [#236]
- Add persistence test to stack contract [#201]
- Add `deploy_with_id` method to NetworkState [#210]

### Changed
- Wrap `NetworkState::modules` in `HostModules` struct [#270]
- Port from `wasmi` to `wasmer` [#245]
- Update dependencies

### Removed
- Remove `block_height` from `NetworkState` [#269]
- Remove Rust toolchain overrides from CI [#229]
- Remove `set_block_height` from NetworkState [#203]

### Fixed
- Fix running tests with `wasmer` [#248]

## [0.6.1] - 2021-07-08
### Added
- Add method to mutate NetworkState bloch_height [#197]

### Changed
- Make `persistence` its own feature (was default feature) [#195]

## [0.6.0] - 2021-07-06
### Added
- Add tests `caller`, `callee-1` and `callee-2` for `dusk-abi::caller` [#185]
- Add `gas_consumed` host function [#174]
- Add `gas_left` host function
- Add instrumentation of Wasm byte-code at deploy time [#115], [#116], see also [#174]
- Add rust toolchain
- Add benchmark for stack contract [#184]
- Add Persistence API for rusk-vm NetworkState [#191]
- Add tests for persistence

### Changed
- Update dependencies
- Update README
- Change `restore` signature to take self ownership

### Fixed
- Fix overflow in `fibonacci` contract

## [0.5.1] - 2021-03-12
### Added
- Add TxVec test case to check transaction of several KiBs

### Changed
- Update `dusk-abi` from `v0.6` to `v0.7`
- Change the tests contracts approach to be simpler
- Change `transact` to store the contract's state before and after

## [0.5.0] - 2021-03-01
### Added
- Add a scoped state for contract's execution [#163]

### Changed
- Change `get_contract_state` to `get_contract_cast_state`
- Add proper error handling where it was still missing
- Update `dusk-abi` to `v0.6`

## [0.4.0] - 2021-02-22
### Added
- Add `get_contract_state` to `NetworkState`
- Add support for Module trait

### Changed
- Rename `hash` contract to `bloch_height`
- Update dependencies

### Removed
- Remove Poseidon Hash from Rusk VM
- Remove `verify_proof` as host function from rusk-vm
- Remove `dusk-abi` directory (it's been added as standalone crate)

## [0.3.1] - 2021-02-17
### Added
- Add ProofVerification ABICall

### Changed
- Update dependencies to latest versions
- Update tests to new rusk-profile API

### Fixed
- Fix execute_circuit const-generic Capacity
- Fix rusk tags
- Fix rusk-profile key obtainment failure

## [0.3.0] - 2021-02-11
### Added
- Add phoenix's ops and add opcode host function (see also [dusk-network/rusk#8], [dusk-network/rusk#28], [dusk-network/dusk-abi#2])
- Add `storage_factorial` example for multiple contract calls
- Add copyright headers
- Add proper license header
- Add Poseidon Sponge Hash function as host function [#131] [see also #123]
- Add `block_height` as host function [#128]

### Changed
- Allow contracts to be called by each other
- Refactor the complete library to support contracts written with canonical data-structures
- Replace Travis with Github Actions [#84]
- Bumb `nstack` to `v0.7` and `microkelvin` to `v0.6`

## [0.2.0] - 2020-04-20
### Changed
- Use https git imports
- Update README

### Removed
- Remove serde dependencies

### Fixed
- Fix Makefile test

## [0.1.2] - 2020-04-20
### Added
- Add marco for Serialize/Deserialize implementation
- Add Makefile and travis integration
- Add rustup target
- Add test for Serialize/Deserialize
- Add benchmark for factorial
- Add phoenix-abi
- Add phoenix_store host function
- Add transfer contract with Approve a TransferFrom functionality
- Add documentation to public interface
- Add LICENSE

### Changed
- Change test.sh to make it working on a multiple system / shell
- Move contracts to 'tests' directory
- Refactor ABI call
- Refactor NetworkState
- Refactor MeteredContract
- Change contract deploy model
- Improve WASM generated size [#34] [#35]
- Update imports to be git-based instead of path-based
- Move contract call enums out of contract files
- Update host functions to work with phoenix v2
- Make Provisioners internals private:
  - Add method to get raw bytes
  - Add method from_bytes
  - Get Provisioners addresses from 'to_bytes'
- Bump Kelvin imports

### Fixed
- Fix Resolver issues

### Removed
- Remove obsolete transfer function

## [0.1.1] - 2020-02-12
### Added
- Add README
- Add 'test_contracts' directory
- Add tool to print out wat from wasm source
- Add basic transaction framework
- Add contract deployment
- Add contract storage
- Add implementation for contract calls
- Add fermion integration
- Add contract signature verification
- Add support for storing arbitrary (de)serializable values in contract state
- Add contract-return implementation
- Add error handling in host_fns
- Add OSX specific linker options
- Add gas-metering
- Add schedule and stack-height
- Add Kelvin integration
- Add NetworkState


## [0.1.0] - 2019-08-02
- Initial

[#374]: https://github.com/dusk-network/rusk-vm/issues/374
[#333]: https://github.com/dusk-network/rusk-vm/issues/333
[#308]: https://github.com/dusk-network/rusk-vm/issues/308
[#304]: https://github.com/dusk-network/rusk-vm/issues/304
[#302]: https://github.com/dusk-network/rusk-vm/issues/302
[#283]: https://github.com/dusk-network/rusk-vm/issues/283
[#270]: https://github.com/dusk-network/rusk-vm/issues/270
[#269]: https://github.com/dusk-network/rusk-vm/issues/269
[#265]: https://github.com/dusk-network/rusk-vm/issues/265
[#263]: https://github.com/dusk-network/rusk-vm/issues/263
[#251]: https://github.com/dusk-network/rusk-vm/issues/251
[#248]: https://github.com/dusk-network/rusk-vm/issues/248
[#247]: https://github.com/dusk-network/rusk-vm/issues/247
[#245]: https://github.com/dusk-network/rusk-vm/issues/245
[#236]: https://github.com/dusk-network/rusk-vm/issues/236
[#205]: https://github.com/dusk-network/rusk-vm/issues/205
[#201]: https://github.com/dusk-network/rusk-vm/issues/201
[#210]: https://github.com/dusk-network/rusk-vm/issues/210
[#203]: https://github.com/dusk-network/rusk-vm/issues/203
[#229]: https://github.com/dusk-network/rusk-vm/issues/229
[#197]: https://github.com/dusk-network/rusk-vm/issues/197
[#195]: https://github.com/dusk-network/rusk-vm/issues/195
[#185]: https://github.com/dusk-network/rusk-vm/issues/185
[#115]: https://github.com/dusk-network/rusk-vm/issues/115
[#116]: https://github.com/dusk-network/rusk-vm/issues/116
[#174]: https://github.com/dusk-network/rusk-vm/issues/174
[#184]: https://github.com/dusk-network/rusk-vm/issues/184
[#191]: https://github.com/dusk-network/rusk-vm/issues/191
[#163]: https://github.com/dusk-network/rusk-vm/issues/163
[#131]: https://github.com/dusk-network/rusk-vm/issues/131
[#123]: https://github.com/dusk-network/rusk-vm/issues/123
[#128]: https://github.com/dusk-network/rusk-vm/issues/128
[#84]: https://github.com/dusk-network/rusk-vm/issues/84
[#83]: https://github.com/dusk-network/rusk-vm/issues/83
[#34]: https://github.com/dusk-network/rusk-vm/issues/34
[#35]: https://github.com/dusk-network/rusk-vm/issues/35

[dusk-network/rusk#28]: https://github.com/dusk-network/rusk/issues/28
[dusk-network/rusk#8]: https://github.com/dusk-network/rusk/issues/8
[dusk-network/dusk-abi#2]: https://github.com/dusk-network/dusk-abi/issues/2

[Unreleased]: https://github.com/dusk-network/rusk-vm/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/dusk-network/rusk-vm/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/dusk-network/rusk-vm/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/dusk-network/rusk-vm/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/dusk-network/rusk-vm/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/dusk-network/rusk-vm/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/dusk-network/rusk-vm/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/dusk-network/rusk-vm/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/dusk-network/rusk-vm/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/dusk-network/rusk-vm/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/dusk-network/rusk-vm/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/dusk-network/rusk-vm/releases/tag/v0.1.0
