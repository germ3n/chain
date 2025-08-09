use {
    crate::vm::{VMTrait, contract::{ContractTrait, ContractValueType}},
    crate::structs::{tx::Tx, account::Address},
    anyhow::Error,
    crate::crypto::hash::Hash,
    std::any::Any,
    std::sync::Arc,
    crate::db::DatabaseTrait,
    mlua::Lua,
    crate::db::open_db,
};

pub struct LVM {
    pub state: Option<LVMState>,
}

impl VMTrait for LVM {
    fn initialize(&mut self, _args: &[&dyn Any]) -> Result<(), Error> {
        let account_store_path = _args[0].downcast_ref::<&str>().expect("Failed to downcast account store path");
        let contract_store_path = _args[1].downcast_ref::<&str>().expect("Failed to downcast contract store path");
        let transaction_store_path = _args[2].downcast_ref::<&str>().expect("Failed to downcast transaction store path");
        self.state = Some(LVMState::new(account_store_path, contract_store_path, transaction_store_path).expect("Failed to initialize LVM"));
        Ok(())
    }

    fn prepare_execution(&self) -> Result<(), Error> {
        unimplemented!("prepare_execution not implemented")
    }

    fn execute_tx(&self, _tx: &Tx) -> Result<(), Error> {
        unimplemented!("execute_tx not implemented")
    }

    fn finalize_execution(&self) -> Result<(), Error> {
        unimplemented!("finalize_execution not implemented")
    }

    fn start_tx(&self) -> Result<(), Error> {
        unimplemented!("start_tx not implemented")
    }

    fn commit_tx(&self) -> Result<(), Error> {
        unimplemented!("commit_tx not implemented")
    }

    fn rollback_tx(&self) -> Result<(), Error> {
        unimplemented!("rollback_tx not implemented")
    }

    fn deploy_contract(&self, _code: &[u8]) -> Result<Address, Error> {
        unimplemented!("deploy_contract not implemented")
    }

    fn get_contract(&self, _hash: &Hash) -> Result<Option<&dyn ContractTrait>, Error> {
        unimplemented!("get_contract not implemented")
    }

    fn get_state(&self) -> Result<&dyn Any, Error> {
        unimplemented!("get_state not implemented")
    }
}

impl LVM {
    pub fn new() -> Self {
        Self { 
            state: None,
        }
    }
}

pub struct LuaEnv {
    pub lua: Lua,
}

pub struct LVMState {
    pub accounts: Arc<Box<dyn DatabaseTrait>>,
    pub contracts: Arc<Box<dyn DatabaseTrait>>,
    pub transactions: Arc<Box<dyn DatabaseTrait>>,
    pub lua: LuaEnv,
}

impl LVMState {
    pub fn new(account_store_path: &str, contract_store_path: &str, transaction_store_path: &str) -> Result<Self, Error> {
        Ok(Self { 
            accounts: open_db(account_store_path)?, 
            contracts: open_db(contract_store_path)?, 
            transactions: open_db(transaction_store_path)?,
            lua: LuaEnv::new()
        })
    }
}

impl LuaEnv {
    pub fn new() -> Self {
        Self { 
            lua: Lua::new(),
        }
    }
}

pub struct Contract {
    pub address: Option<Address>,
    pub code: Option<Vec<u8>>,
}

impl ContractTrait for Contract {
    fn new(data: &[&dyn Any]) -> Self {
        let address = data[0].downcast_ref::<Address>().unwrap();
        let code = data[1].downcast_ref::<&[u8]>().unwrap();
        Self { 
            address: Some(address.clone()),
            code: Some(code.to_vec()),
        }
    }

    fn get_code(&self) -> Option<&[u8]> {
        match self.code {
            Some(ref code) => Some(code),
            None => None,
        }
    }

    fn get_address(&self) -> Option<&Address> {
        self.address.as_ref()
    }

    fn upgrade(&self, _code: Vec<u8>) -> Result<(), Error> {
        unimplemented!("upgrade not implemented")
    }

    fn call_function(&self, _function: Hash, _args: &[&ContractValueType]) -> Result<Vec<u8>, Error> {
        unimplemented!("call_function not implemented")
    }

    fn get_from_storage(&self, _key: &[u8]) -> Result<Vec<u8>, Error> {
        unimplemented!("get_from_storage not implemented")
    }

    fn put_to_storage(&self, _key: &[u8], _value: &[u8]) -> Result<(), Error> {
        unimplemented!("put_to_storage not implemented")
    }
}