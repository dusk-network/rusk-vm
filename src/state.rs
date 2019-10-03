use std::collections::HashMap;

use dusk_abi::{encoding, H256};
use failure::{bail, format_err, Error};
use serde::Serialize;
use wasmi::{ExternVal, ImportsBuilder, ModuleInstance};

use crate::contract::Contract;
use crate::digest::Digest;
use crate::host_fns::{CallContext, HostImportResolver};

pub type Storage = HashMap<Vec<u8>, Vec<u8>>;

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
    pub fn genesis(contract: Contract, value: u128) -> Self {
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
        state.deploy_contract(contract);
        state
    }

    pub fn genesis_id(&self) -> &H256 {
        &self.genesis_id
    }

    // Deploys contract to the network state and runs the deploy function
    pub fn deploy_contract(&mut self, contract: Contract) -> Result<(), Error> {
        let id = contract.digest();

        let state =
            self.contracts.entry(id).or_insert(ContractState::default());

        if state.contract.bytecode().len() == 0 {
            state.contract = contract
        }

        let module = wasmi::Module::from_buffer(state.contract.bytecode())?;
        Self::invoke_bytecode(
            &module,
            &id,
            "deploy",
            &[],
            state.storage_mut(),
        )?;

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

    fn invoke_bytecode(
        module: &wasmi::Module,
        caller: &H256,
        call: &str,
        call_data: &[u8],
        storage: &mut Storage,
    ) -> Result<(), Error> {
        let imports =
            ImportsBuilder::new().with_resolver("env", &HostImportResolver);

        let instance =
            ModuleInstance::new(&module, &imports)?.assert_no_start();

        // Get memory reference for call
        match instance.export_by_name("memory") {
            Some(ExternVal::Memory(memref)) => {
                let mut externals =
                    CallContext::new(&memref, caller, storage, call_data);
                // Run contract initialization
                instance.invoke_export(call, &[], &mut externals)?;
                Ok(())
            }
            _ => bail!("No memory available"),
        }
    }

    pub fn call_contract<C: Serialize>(
        &mut self,
        contract_id: &H256,
        call: &C,
    ) -> Result<(), Error> {
        let state = self
            .get_contract_state_mut(contract_id)
            .ok_or_else(|| format_err!("no such contract"))?;

        let mut buf = [0u8; 1024];

        let slice = encoding::encode(call, &mut buf)?;

        let module = wasmi::Module::from_buffer(state.contract().bytecode())?;
        Self::invoke_bytecode(
            &module,
            // In top level call, caller is "self"
            contract_id,
            "call",
            slice,
            state.storage_mut(),
        )?;

        Ok(())
    }
}
