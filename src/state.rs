use std::collections::HashMap;

use ethereum_types::U256;
use failure::{bail, format_err, Error};
use wasmi::{ExternVal, ImportsBuilder, ModuleInstance};

use crate::host_fns::{HostExternals, HostImportResolver};
use crate::prepare_module::prepare_module;
use crate::transaction::Transaction;
use crate::wallet::ManagedAccount;

#[derive(Default, Debug)]
struct Block {
    height: u128,
    transactions: Vec<Transaction>,
}

#[derive(Default, Debug)]
pub struct Account {
    nonce: u128,
    balance: u128,
}

impl Account {
    pub fn balance(&self) -> u128 {
        self.balance
    }

    pub fn nonce(&self) -> u128 {
        self.nonce
    }

    pub fn balance_mut(&mut self) -> &mut u128 {
        &mut self.balance
    }

    pub fn nonce_mut(&mut self) -> &mut u128 {
        &mut self.nonce
    }
}

#[derive(Debug)]
pub struct Contract {
    balance: u128,
    bytecode: Vec<u8>,
    storage: HashMap<U256, U256>,
}

impl Contract {
    pub fn storage(&self) -> &HashMap<U256, U256> {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut HashMap<U256, U256> {
        &mut self.storage
    }

    pub fn bytecode(&self) -> &[u8] {
        &self.bytecode
    }
}

#[derive(Default, Debug)]
pub struct NetworkState {
    accounts: HashMap<U256, Account>,
    contracts: HashMap<U256, Contract>,
    blocks: Vec<Block>,
    queue: Vec<Transaction>,
}

impl NetworkState {
    pub fn genesis(mint: &ManagedAccount) -> Self {
        let account_id = mint.id();
        let mut accounts = HashMap::new();
        accounts.insert(
            account_id.clone(),
            Account {
                balance: 1_000_000,
                nonce: 0,
            },
        );
        NetworkState {
            accounts,
            contracts: HashMap::new(),
            blocks: vec![],
            queue: vec![],
        }
    }

    pub fn get_contract(&self, contract_id: &U256) -> Option<&Contract> {
        self.contracts.get(contract_id)
    }

    pub fn get_contract_mut(
        &mut self,
        contract_id: &U256,
    ) -> Option<&mut Contract> {
        self.contracts.get_mut(contract_id)
    }

    pub fn get_account(&self, account_id: &U256) -> Option<&Account> {
        self.accounts.get(account_id)
    }

    // Gets a mutable reference to the account id, creating it if it does not already exist
    pub fn get_account_mut(&mut self, account_id: &U256) -> &mut Account {
        self.accounts.entry(*account_id).or_default()
    }

    pub fn mint_block(&mut self) -> Result<(), Error> {
        let mut block = vec![];
        let mut queue = std::mem::replace(&mut self.queue, vec![]);
        for tcn in queue.drain(..) {
            if tcn.valid(self) {
                tcn.apply(self)?;
                block.push(tcn);
            }
        }
        self.append_transaction_block(block);
        Ok(())
    }

    fn append_transaction_block(&mut self, _transactions: Vec<Transaction>) {
        ()
    }

    pub fn queue_transaction(&mut self, transaction: Transaction) {
        self.queue.push(transaction)
    }

    fn invoke_code(
        module: &wasmi::Module,
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
                let mut externals = HostExternals::new(&memref, storage);
                // Run contract initialization
                instance.invoke_export(call, &[], &mut externals)?;
                Ok(())
            }
            _ => bail!("No memory available"),
        }
    }

    pub fn deploy_contract(
        &mut self,
        bytecode: &[u8],
        contract_id: &U256,
        balance: u128,
    ) -> Result<(), Error> {
        let (ctor, bytecode) = prepare_module(bytecode)?;

        let mut storage = HashMap::new();
        Self::invoke_code(&ctor, "deploy", &mut storage)?;

        let contract = Contract {
            balance,
            bytecode,
            storage,
        };

        self.contracts.insert(contract_id.clone(), contract);

        Ok(())
    }

    pub fn call_contract(
        &mut self,
        data: &[u8],
        contract_id: &U256,
        value: u128,
    ) -> Result<(), Error> {
        let contract = self
            .get_contract_mut(contract_id)
            .ok_or_else(|| format_err!("no such contract"))?;

        let module = wasmi::Module::from_buffer(contract.bytecode())?;
        Self::invoke_code(&module, "call", contract.storage_mut())?;

        Ok(())
    }
}
