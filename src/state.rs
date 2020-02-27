use std::io;
use std::marker::PhantomData;

use crate::VMError;
use dusk_abi::{encoding, ContractCall, H256};
use kelvin::{ByteHash, Content, Map as _, Sink, Source, ValRef, ValRefMut};
use kelvin_radix::DefaultRadixMap as RadixMap;
use serde::Deserialize;

use crate::contract::MeteredContract;
use crate::gas::GasMeter;
use crate::host_fns::{CallContext, CallKind, Resolver};

pub type Storage<H> = RadixMap<H256, Vec<u8>, H>;

#[derive(Default, Clone)]
pub struct ContractState<H: ByteHash> {
    balance: u128,
    code: MeteredContract,
    nonce: u64,
    storage: Storage<H>,
}

impl<H: ByteHash> ContractState<H> {
    pub fn balance(&self) -> u128 {
        self.balance
    }

    pub fn balance_mut(&mut self) -> &mut u128 {
        &mut self.balance
    }

    pub fn storage(&self) -> &Storage<H> {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut Storage<H> {
        &mut self.storage
    }

    pub fn contract(&self) -> &MeteredContract {
        &self.code
    }
}

#[derive(Clone, Default)]
pub struct NetworkState<S, H: ByteHash> {
    contracts: RadixMap<H256, ContractState<H>, H>,
    _marker: PhantomData<S>,
}

impl<S: Resolver<H>, H: ByteHash> NetworkState<S, H> {
    pub fn deploy(&mut self, code: &[u8]) -> Result<H256, VMError> {
        let metered = MeteredContract::new(code)?;

        let code_hash = H256::from_bytes(H::hash(&metered.bytecode()).as_ref());

        let contract = ContractState {
            code: metered,
            ..Default::default()
        };

        self.contracts.insert(code_hash.clone(), contract)?;
        Ok(code_hash)
    }

    // Deploys contract to the network state and runs the deploy function
    pub fn deploy_contract(
        &mut self,
        _contract: MeteredContract,
        _gas_meter: &mut GasMeter,
    ) -> Result<(), VMError> {
        unimplemented!()

        // let id = contract.digest();

        // if self.contracts.get(&id)?.is_none() {
        //     self.contracts.insert(id, ContractState::default())?;
        // }

        // {
        //     let mut state = self
        //         .contracts
        //         .get_mut(&id)?
        //         .expect("Assured populated above");

        //     if state.contract.bytecode().is_empty() {
        //         state.contract = contract
        //     }
        // }

        // let deploy_buffer = [0u8; CALL_DATA_SIZE];

        // let mut context = CallContext::new(self, gas_meter);
        // context.call(id, deploy_buffer, CallKind::Deploy)?;

        // Ok(())
    }

    pub fn get_contract_state(
        &self,
        contract_id: &H256,
    ) -> Result<Option<impl ValRef<ContractState<H>>>, VMError> {
        self.contracts.get(contract_id).map_err(Into::into)
    }

    pub fn get_contract_state_mut(
        &mut self,
        contract_id: &H256,
    ) -> Result<Option<impl ValRefMut<ContractState<H>>>, VMError> {
        self.contracts.get_mut(contract_id).map_err(Into::into)
    }

    pub fn get_contract_state_mut_or_default(
        &mut self,
        id: &H256,
    ) -> Result<impl ValRefMut<ContractState<H>>, VMError> {
        if self.contracts.get(id)?.is_none() {
            self.contracts.insert(*id, ContractState::default())?;
        }

        Ok(self.contracts.get_mut(id)?.expect("Assured above"))
    }

    pub fn call_contract<R: for<'de> Deserialize<'de>>(
        &mut self,
        target: H256,
        call: ContractCall<R>,
        gas_meter: &mut GasMeter,
    ) -> Result<R, VMError> {
        let mut context = CallContext::new(self, gas_meter);
        let data = call.into_data();
        let data_return = context.call(target, data, CallKind::Call)?;
        let decoded = encoding::decode(&data_return)?;
        Ok(decoded)
    }
}

impl<H: ByteHash> Content<H> for ContractState<H> {
    fn persist(&mut self, sink: &mut Sink<H>) -> io::Result<()> {
        self.balance.persist(sink)?;
        self.nonce.persist(sink)?;
        self.code.persist(sink)?;
        self.storage.persist(sink)
    }

    fn restore(source: &mut Source<H>) -> io::Result<Self> {
        Ok(ContractState {
            balance: u128::restore(source)?,
            nonce: u64::restore(source)?,
            code: MeteredContract::restore(source)?,
            storage: Storage::restore(source)?,
        })
    }
}

impl<S: 'static + Resolver<H>, H: ByteHash> Content<H> for NetworkState<S, H> {
    fn persist(&mut self, sink: &mut Sink<H>) -> io::Result<()> {
        self.contracts.persist(sink)
    }

    fn restore(source: &mut Source<H>) -> io::Result<Self> {
        Ok(NetworkState {
            contracts: RadixMap::restore(source)?,
            _marker: PhantomData,
        })
    }
}
