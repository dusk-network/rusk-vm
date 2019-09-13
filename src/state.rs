use std::collections::HashMap;

use ethereum_types::U256;
use failure::{bail, format_err, Error};
use wasmi::{ExternVal, ImportsBuilder, ModuleInstance};

use crate::contract_builder::WasmBytecode;
use crate::digest::Digest;
use crate::host_fns::{CallContext, HostImportResolver};
use crate::wallet::ManagedAccount;

use crate::contract_code;

#[derive(Default, Debug, Clone)]
pub struct Contract {
    balance: u128,
    bytecode: Vec<u8>,
    storage: HashMap<U256, U256>,
}

impl Contract {
    pub fn balance(&self) -> u128 {
        self.balance
    }

    pub fn balance_mut(&mut self) -> &mut u128 {
        &mut self.balance
    }

    pub fn storage(&self) -> &HashMap<U256, U256> {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut HashMap<U256, U256> {
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
#[derive(Default, Debug, Clone)]
pub struct NetworkState(HashMap<U256, Contract>);

impl NetworkState {
    pub fn genesis() -> Self {
        let default_account_id =
            (&contract_code!("default_account")[..]).digest();
        let mut contracts = HashMap::new();
        contracts.insert(
            default_account_id,
            Contract {
                bytecode: vec![],
                balance: 1_000_000,
                storage: HashMap::default(),
            },
        );
        NetworkState(contracts)
    }

    // Deploys contract to the network state and runs the deploy function
    pub fn deploy_contract(
        &mut self,
        bytecode: WasmBytecode,
    ) -> Result<(), Error> {
        let id = bytecode.digest();

        let contract = self.0.entry(id).or_insert(Contract {
            bytecode: vec![],
            balance: 0,
            storage: HashMap::default(),
        });

        if contract.bytecode().len() == 0 {
            *contract.bytecode_mut() = bytecode.into_bytecode();
        }

        let module = wasmi::Module::from_buffer(contract.bytecode())?;
        Self::invoke_bytecode(&module, id, "deploy", contract.storage_mut())?;

        Ok(())
    }

    pub fn get_contract(&self, contract_id: &U256) -> Option<&Contract> {
        self.0.get(contract_id)
    }

    pub fn get_contract_mut(
        &mut self,
        contract_id: &U256,
    ) -> Option<&mut Contract> {
        self.0.get_mut(contract_id)
    }

    fn invoke_bytecode(
        module: &wasmi::Module,
        caller: U256,
        call: &str,
        storage: &mut HashMap<U256, U256>,
    ) -> Result<(), Error> {
        let imports =
            ImportsBuilder::new().with_resolver("env", &HostImportResolver);

        let instance =
            ModuleInstance::new(&module, &imports)?.assert_no_start();

        // Get memory reference for call
        match instance.export_by_name("memory") {
            Some(ExternVal::Memory(memref)) => {
                let mut externals = CallContext::new(&memref, caller, storage);
                // Run contract initialization
                instance.invoke_export(call, &[], &mut externals)?;
                Ok(())
            }
            _ => bail!("No memory available"),
        }
    }

    pub fn call_contract(
        &mut self,
        caller: U256,
        _data: &[u8],
        contract_id: &U256,
        _value: u128,
    ) -> Result<(), Error> {
        let contract = self
            .get_contract_mut(contract_id)
            .ok_or_else(|| format_err!("no such contract"))?;

        let module = wasmi::Module::from_buffer(contract.bytecode())?;
        Self::invoke_bytecode(&module, caller, "call", contract.storage_mut())?;

        Ok(())
    }
}
