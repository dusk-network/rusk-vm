use std::io;

use crate::VMError;
use dusk_abi::{encoding, ContractCall, CALL_DATA_SIZE, H256};
use kelvin::{Blake2b, Content, Map as _, Sink, Source, ValRef, ValRefMut};
use kelvin_radix::DefaultRadixMap as RadixMap;
use serde::Deserialize;

use crate::contract::Contract;
use crate::digest::Digest;
use crate::gas::GasMeter;
use crate::host_fns::{CallContext, CallKind, DynamicResolver};

pub type Storage = RadixMap<H256, Vec<u8>, Blake2b>;

#[derive(Default, Clone)]
pub struct ContractState {
    balance: u128,
    contract: Contract,
    storage: Storage,
}

impl ContractState {
    pub fn balance(&self) -> u128 {
        self.balance
    }

    pub fn balance_mut(&mut self) -> &mut u128 {
        &mut self.balance
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut Storage {
        &mut self.storage
    }

    pub fn contract(&self) -> &Contract {
        &self.contract
    }

    pub fn contract_mut(&mut self) -> &mut Contract {
        &mut self.contract
    }
}

// Clone is obviously relatively expensive in the mock implementation
// but it will be using persistent datastructures in production
#[derive(Clone, Default)]
pub struct NetworkState<S> {
    genesis_id: H256,
    contracts: RadixMap<H256, ContractState, Blake2b>,
    resolver: S,
}

impl<S: DynamicResolver> NetworkState<S> {
    pub fn genesis(
        contract: Contract,
        value: u128,
        resolver: &S,
    ) -> Result<Self, VMError> {
        let genesis_id = contract.digest();
        let mut contracts = RadixMap::new();
        contracts.insert(
            genesis_id.clone(),
            ContractState {
                balance: value,
                ..Default::default()
            },
        )?;

        let mut state = NetworkState {
            genesis_id,
            contracts,
            resolver: S::default(),
        };
        state.deploy_contract(contract, resolver)?;
        Ok(state)
    }

    pub fn genesis_id(&self) -> &H256 {
        &self.genesis_id
    }

    pub fn resolver(&self) -> &S {
        &self.resolver
    }

    // Deploys contract to the network state and runs the deploy function
    pub fn deploy_contract(
        &mut self,
        contract: Contract,
        resolver: &S,
    ) -> Result<(), VMError> {
        let id = contract.digest();

        if self.contracts.get(&id)?.is_none() {
            self.contracts.insert(id, ContractState::default())?;
        }

        {
            let mut state = self
                .contracts
                .get_mut(&id)?
                .expect("Assured populated above");

            if state.contract.bytecode().is_empty() {
                state.contract = contract
            }
        }

        let deploy_buffer = [0u8; CALL_DATA_SIZE];

        let mut context = CallContext::new(self, resolver);
        context.call(id, deploy_buffer, CallKind::Deploy)?;

        Ok(())
    }

    pub fn get_contract_state(
        &self,
        contract_id: &H256,
    ) -> Result<Option<impl ValRef<ContractState>>, VMError> {
        self.contracts.get(contract_id).map_err(Into::into)
    }

    pub fn get_contract_state_mut(
        &mut self,
        contract_id: &H256,
    ) -> Result<Option<impl ValRefMut<ContractState>>, VMError> {
        self.contracts.get_mut(contract_id).map_err(Into::into)
    }

    pub fn get_contract_state_mut_or_default(
        &mut self,
        id: &H256,
    ) -> Result<impl ValRefMut<ContractState>, VMError> {
        if self.contracts.get(id)?.is_none() {
            self.contracts.insert(*id, ContractState::default())?;
        }

        Ok(self.contracts.get_mut(id)?.expect("Assured above"))
    }

    pub fn call_contract<R: for<'de> Deserialize<'de>>(
        &mut self,
        target: H256,
        call: ContractCall<R>,
        resolver: &S,
    ) -> Result<R, VMError> {
        let mut context = CallContext::new(self, resolver);
        let data = call.into_data();
        let data_return = context.call(target, data, CallKind::Call)?;
        let decoded = encoding::decode(&data_return)?;
        Ok(decoded)
    }

    pub fn call_contract_with_limit<R: for<'de> Deserialize<'de>>(
        &mut self,
        target: H256,
        call: ContractCall<R>,
        gas_meter: &mut GasMeter,
        resolver: &S,
    ) -> Result<R, VMError> {
        let mut context = CallContext::with_limit(self, gas_meter, resolver);
        let data = call.into_data();
        let data_return = context.call(target, data, CallKind::Call)?;
        let decoded = encoding::decode(&data_return)?;
        Ok(decoded)
    }
}

impl Content<Blake2b> for ContractState {
    fn persist(&mut self, sink: &mut Sink<Blake2b>) -> io::Result<()> {
        self.balance.persist(sink)?;
        self.contract.persist(sink)?;
        self.storage.persist(sink)
    }

    fn restore(source: &mut Source<Blake2b>) -> io::Result<Self> {
        Ok(ContractState {
            balance: u128::restore(source)?,
            contract: Contract::restore(source)?,
            storage: Storage::restore(source)?,
        })
    }
}

impl<S: 'static + DynamicResolver> Content<Blake2b> for NetworkState<S> {
    fn persist(&mut self, sink: &mut Sink<Blake2b>) -> io::Result<()> {
        self.genesis_id.persist(sink)?;
        self.contracts.persist(sink)
    }

    fn restore(source: &mut Source<Blake2b>) -> io::Result<Self> {
        Ok(NetworkState {
            genesis_id: H256::restore(source)?,
            contracts: RadixMap::restore(source)?,
            resolver: S::default(),
        })
    }
}
