use {
    crate::vm::{VMTrait, contract::ContractTrait},
    crate::structs::{tx::Tx, account::Address},
    anyhow::Error,
    crate::crypto::hash::Hash,
    std::any::Any,
    std::sync::Arc,
    crate::db::DatabaseTrait,
    mlua::Lua,
    std::marker::PhantomData,
};

pub struct LVM<D: DatabaseTrait> {
    pub state: Option<LVMState<D>>,
}

impl<D: DatabaseTrait> VMTrait<D> for LVM<D> {
    fn init(&self) -> Result<(), Error> {
        Ok(())
    }

    fn prepare_execution(&self) -> Result<(), Error> {
        Err(Error::msg("Not implemented"))
    }

    fn execute_tx(&self, _tx: &Tx) -> Result<(), Error> {
        Err(Error::msg("Not implemented"))
    }

    fn finalize_execution(&self) -> Result<(), Error> {
        Err(Error::msg("Not implemented"))
    }

    fn start_tx(&self) -> Result<(), Error> {
        Err(Error::msg("Not implemented"))
    }

    fn commit_tx(&self) -> Result<(), Error> {
        Err(Error::msg("Not implemented"))
    }

    fn rollback_tx(&self) -> Result<(), Error> {
        Err(Error::msg("Not implemented"))
    }

    fn deploy_contract(&self, _code: &[u8]) -> Result<Hash, Error> {
        Err(Error::msg("Not implemented"))
    }

    fn get_contract(&self, _hash: &Hash) -> Result<Option<&dyn ContractTrait<D>>, Error> {
        Err(Error::msg("Not implemented"))
    }

    fn get_state(&self) -> Result<&dyn Any, Error> {
        Err(Error::msg("Not implemented"))
    }
}

impl<D: DatabaseTrait> LVM<D> {
    pub fn new(db: Arc<D>) -> Self {
        Self { 
            state: Some(LVMState::new(db)),
        }
    }
}

pub struct LuaEnv {
    pub lua: Lua,
}

pub struct LVMState<D: DatabaseTrait> {
    pub accounts: Arc<D>,
    pub contracts: Arc<D>,
    pub transactions: Arc<D>,
    pub lua: LuaEnv,
}

impl<D: DatabaseTrait> LVMState<D> {
    pub fn new(db: Arc<D>) -> Self {
        Self { 
            accounts: db.clone(), 
            contracts: db.clone(), 
            transactions: db.clone(), 
            lua: LuaEnv::new()
        }
    }
}

impl LuaEnv {
    pub fn new() -> Self {
        Self { 
            lua: Lua::new(),
        }
    }
}

pub struct Contract<D: DatabaseTrait> {
    pub address: Address,
    pub code: Arc<Vec<u8>>,
    pub last_modified: Option<u64>,
    pub phantom: PhantomData<D>,
}

impl<D: DatabaseTrait + Send + Sync + 'static> ContractTrait<D> for Contract<D> {
    fn new(data: &dyn Any) -> Self {
        let (address, code, last_modified) = data.downcast_ref::<(Address, Arc<Vec<u8>>, Option<u64>)>().expect("Invalid contract data");
        Self { 
            address: address.clone(),
            code: code.clone(),
            last_modified: *last_modified,
            phantom: PhantomData,
        }
    }

    fn get_code(&self) -> Result<Arc<Vec<u8>>, Error> {
        Ok(self.code.clone())
    }

    fn get_address(&self) -> &Address {
        &self.address
    }

    fn get_last_modified(&self) -> Option<u64> {
        self.last_modified
    }
}