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

pub mod numlib;

pub struct LVM {
    pub state: Option<LVMState>,
}

impl VMTrait for LVM {
    fn initialize(&mut self, args: &[&dyn Any]) -> Result<(), Error> {
        let account_store_path = args[0].downcast_ref::<String>().expect("Failed to downcast account store path");
        let contract_store_path = args[1].downcast_ref::<String>().expect("Failed to downcast contract store path");
        let transaction_store_path = args[2].downcast_ref::<String>().expect("Failed to downcast transaction store path");
        self.state = Some(LVMState::new(account_store_path, contract_store_path, transaction_store_path)?);
        Ok(())
    }

    fn prepare_execution(&self) -> Result<(), Error> {
        //unimplemented!("prepare_execution not implemented")
        Ok(())
    }

    fn execute_tx(&self, _tx: &Tx) -> Result<(), Error> {
        //unimplemented!("execute_tx not implemented")
        Ok(())
    }

    fn finalize_execution(&self) -> Result<(), Error> {
        //unimplemented!("finalize_execution not implemented")
        Ok(())
    }

    fn start_tx(&self) -> Result<(), Error> {
        //unimplemented!("start_tx not implemented")
        Ok(())
    }

    fn commit_tx(&self) -> Result<(), Error> {
        //unimplemented!("commit_tx not implemented")
        Ok(())
    }

    fn rollback_tx(&self) -> Result<(), Error> {
        //unimplemented!("rollback_tx not implemented")
        Ok(())
    }

    fn deploy_contract(&self, _code: &[u8]) -> Result<Address, Error> {
        //unimplemented!("deploy_contract not implemented")
        Ok(Address::Null)
    }

    fn get_contract(&self, _hash: &Hash) -> Result<Option<&dyn ContractTrait>, Error> {
        //unimplemented!("get_contract not implemented")
        Ok(None)
    }

    fn get_state(&self) -> Result<&dyn Any, Error> {
        //unimplemented!("get_state not implemented")
        Ok(self.state.as_ref().unwrap())
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
    pub fn new(account_store_path: &String, contract_store_path: &String, transaction_store_path: &String) -> Result<Self, Error> {
        let state = Self { 
            accounts: open_db(account_store_path.to_owned())?, 
            contracts: open_db(contract_store_path.to_owned())?, 
            transactions: open_db(transaction_store_path.to_owned())?,
            lua: LuaEnv::new()
        };
        
        numlib::register_all_numeric_libraries(&state.lua.lua)?;

        info!("LVM state initialized");
        Ok(state)
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
    fn from_code(data: &[&dyn Any]) -> Self {
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