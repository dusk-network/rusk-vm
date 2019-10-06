use std::collections::HashMap;

use dusk_abi::{ContractCall, CALL_DATA_SIZE, H256, STORAGE_KEY_SIZE};
use failure::Error;
use serde::Deserialize;

use crate::contract::Contract;
use crate::digest::Digest;
use crate::host_fns::{CallContext, CallKind};

pub type Storage = HashMap<[u8; STORAGE_KEY_SIZE], Vec<u8>>;

#[derive(Default, Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct NetworkState {
    genesis_id: H256,
    contracts: HashMap<H256, ContractState>,
}

impl NetworkState {
    pub fn genesis(contract: Contract, value: u128) -> Result<Self, Error> {
        let genesis_id = contract.digest();
        let mut contracts = HashMap::new();
        contracts.insert(
            genesis_id.clone(),
            ContractState {
                balance: value,
                ..Default::default()
            },
        );
        let mut state = NetworkState {
            genesis_id,
            contracts,
        };
        state.deploy_contract(contract)?;
        Ok(state)
    }

    pub fn genesis_id(&self) -> &H256 {
        &self.genesis_id
    }

    // Deploys contract to the network state and runs the deploy function
    pub fn deploy_contract(&mut self, contract: Contract) -> Result<(), Error> {
        let id = contract.digest();

        let mut state =
            self.contracts.entry(id).or_insert(ContractState::default());

        if state.contract.bytecode().len() == 0 {
            state.contract = contract
        }
        let mut deploy_buffer = [0u8; CALL_DATA_SIZE];

        let mut context = CallContext::new(self);
        context.call(id, &mut deploy_buffer, CallKind::Deploy)?;

        Ok(())
    }

    pub fn get_contract_state(
        &self,
        contract_id: &H256,
    ) -> Option<&ContractState> {
        self.contracts.get(contract_id)
    }

    pub fn get_contract_state_mut(
        &mut self,
        contract_id: &H256,
    ) -> Option<&mut ContractState> {
        self.contracts.get_mut(contract_id)
    }

    pub fn get_contract_state_mut_or_default(
        &mut self,
        contract_id: &H256,
    ) -> &mut ContractState {
        self.contracts
            .entry(*contract_id)
            .or_insert(ContractState::default())
    }

    pub fn call_contract<'de, R: Deserialize<'de>>(
        &mut self,
        target: H256,
        mut call: ContractCall<R>,
    ) -> Result<R, Error> {
        let mut context = CallContext::new(self);
        let data = call.data_mut();
        context.call(target, data, CallKind::Call)?;
        unimplemented!()
    }
}
