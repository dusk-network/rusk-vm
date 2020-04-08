use std::io;
use std::marker::PhantomData;

use dataview::Pod;
use dusk_abi::H256;
use kelvin::{ByteHash, Content, Map, Sink, Source, ValRef, ValRefMut};
use kelvin_radix::DefaultRadixMap as RadixMap;

use crate::call_context::{CallContext, Resolver};
use crate::contract::{Contract, MeteredContract};
use crate::gas::GasMeter;
use crate::VMError;

pub type Storage<H> = RadixMap<H256, Vec<u8>, H>;

#[derive(Clone)]
pub struct ContractState<H: ByteHash> {
    balance: u128,
    code: MeteredContract,
    nonce: u64,
    storage: Storage<H>,
}

impl<H: ByteHash> ContractState<H> {
    pub fn empty() -> Self {
        ContractState {
            code: MeteredContract::empty(),
            balance: 0,
            nonce: 0,
            storage: Default::default(),
        }
    }

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

    pub fn contract_mut(&mut self) -> &mut MeteredContract {
        &mut self.code
    }
}

/// The main network state, includes the full state of contracts.
#[derive(Clone, Default)]
pub struct NetworkState<S, H: ByteHash> {
    contracts: RadixMap<H256, ContractState<H>, H>,
    _marker: PhantomData<S>,
}

impl<S: Resolver<H>, H: ByteHash> NetworkState<S, H> {
    /// Deploys a contract to the state, returns the address of the created contract
    /// or an error
    pub fn deploy(&mut self, contract: Contract) -> Result<H256, VMError> {
        let metered = contract.build()?;

        let code = metered.bytecode();

        let code_hash = H256::from_bytes(H::hash(&code).as_ref());

        let state = ContractState {
            code: metered,
            balance: 0,
            nonce: 0,
            storage: Default::default(),
        };

        self.contracts.insert(code_hash.clone(), state)?;
        Ok(code_hash)
    }

    /// Returns a reference to the specified contracts state
    pub fn get_contract_state(
        &self,
        contract_id: &H256,
    ) -> Result<Option<impl ValRef<ContractState<H>>>, VMError> {
        self.contracts.get(contract_id).map_err(Into::into)
    }

    /// Returns a mutable reference to the specified contracts state
    pub fn get_contract_state_mut(
        &mut self,
        contract_id: &H256,
    ) -> Result<Option<impl ValRefMut<ContractState<H>>>, VMError> {
        self.contracts.get_mut(contract_id).map_err(Into::into)
    }

    /// Returns a mutable reference to the specified contracts state, or to a newly created
    /// contract
    pub fn get_contract_state_mut_or_default(
        &mut self,
        id: &H256,
    ) -> Result<impl ValRefMut<ContractState<H>>, VMError> {
        if self.contracts.get(id)?.is_none() {
            self.contracts.insert(*id, ContractState::empty())?;
        }

        Ok(self.contracts.get_mut(id)?.expect("Assured above"))
    }

    /// Call the contract at address `target`
    pub fn call_contract<A: Pod, R: Pod>(
        &mut self,
        target: H256,
        argument: A,
        gas_meter: &mut GasMeter,
    ) -> Result<R, VMError> {
        let mut ret = R::zeroed();
        let mut context =
            CallContext::new(self, gas_meter, &argument, &mut ret);
        context.call(target, 0, 0, 0, 0)?;
        Ok(ret)
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
