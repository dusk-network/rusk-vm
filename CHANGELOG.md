# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Add CHANGELOG [#236]
- Add persistence test to stack contract [#201]
- Add `deploy_with_id` method to NetworkState [#210]

### Changed
- Update dependencies

### Removed
- Remove `set_block_height` from NetworkState [#203]

### Fixed
- Fix failing CI [#229]

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
- 

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

[#236]: https://github.com/dusk-network/rusk-vm/issues/236
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
