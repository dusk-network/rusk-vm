use std::collections::HashMap;

use dusk_abi::types::H256;
use failure::{bail, format_err, Error};
use wasmi::{ExternVal, ImportsBuilder, ModuleInstance};

use crate::contract_builder::WasmBytecode;
use crate::digest::Digest;
use crate::host_fns::{CallContext, HostImportResolver};

use crate::contract_code;

#[derive(Default, Debug, Clone)]
pub struct Contract {
    balance: u128,
    bytecode: Vec<u8>,
    storage: HashMap<H256, H256>,
}

impl Contract {
    pub fn balance(&self) -> u128 {
        self.balance
    }

    pub fn balance_mut(&mut self) -> &mut u128 {
        &mut self.balance
    }

    pub fn storage(&self) -> &HashMap<H256, H256> {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut HashMap<H256, H256> {
        &mut self.storage
    }

    pub fn bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    pub fn bytecode_mut(&mut self) -> &mut Vec<u8> {
        &mut self.bytecode
    }
}

// Clone is obviously relatively expensive in the mock implementation
// but it will be using persistent datastructures in production
#[derive(Debug, Clone)]
pub struct NetworkState {
    genesis_id: H256,
    contracts: HashMap<H256, Contract>,
}

impl NetworkState {
    pub fn genesis(bytecode: &[u8], value: u128) -> Self {
        let mut genesis_code = vec![];
        genesis_code.extend_from_slice(bytecode);

        let genesis_id = (&genesis_code[..]).digest();
        let mut contracts = HashMap::new();
        contracts.insert(
            genesis_id.clone(),
            Contract {
                bytecode: genesis_code,
                balance: 1_000_000,
                storage: HashMap::default(),
            },
        );
        NetworkState {
            genesis_id,
            contracts,
        }
    }

    pub fn genesis_id(&self) -> &H256 {
        &self.genesis_id
    }

    // Deploys contract to the network state and runs the deploy function
    pub fn deploy_contract(
        &mut self,
        bytecode: WasmBytecode,
    ) -> Result<(), Error> {
        let id = bytecode.digest();

        let contract = self.contracts.entry(id).or_insert(Contract {
            bytecode: vec![],
            balance: 0,
            storage: HashMap::default(),
        });

        if contract.bytecode().len() == 0 {
            *contract.bytecode_mut() = bytecode.into_bytecode();
        }

        let module = wasmi::Module::from_buffer(contract.bytecode())?;
        Self::invoke_bytecode(
            &module,
            &id,
            "deploy",
            &[],
            contract.storage_mut(),
        )?;

        Ok(())
    }

    pub fn get_contract(&self, contract_id: &H256) -> Option<&Contract> {
        self.contracts.get(contract_id)
    }

    pub fn get_contract_mut(
        &mut self,
        contract_id: &H256,
    ) -> Option<&mut Contract> {
        self.contracts.get_mut(contract_id)
    }

    fn invoke_bytecode(
        module: &wasmi::Module,
        caller: &H256,
        call: &str,
        call_data: &[u8],
        storage: &mut HashMap<H256, H256>,
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

    pub fn call_contract(
        &mut self,
        contract_id: &H256,
        _value: u128,
        data: &[u8],
    ) -> Result<(), Error> {
        let contract = self
            .get_contract_mut(contract_id)
            .ok_or_else(|| format_err!("no such contract"))?;

        let module = wasmi::Module::from_buffer(contract.bytecode())?;
        Self::invoke_bytecode(
            &module,
            // In top level call, caller is "self"
            contract_id,
            "call",
            data,
            contract.storage_mut(),
        )?;

        Ok(())
    }
}
