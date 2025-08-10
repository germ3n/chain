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
    mlua::prelude::*,
    mlua::{Function, Value},
};


pub mod numlib;
pub mod compiler;

pub struct LVM {
    pub state: Option<Arc<LVMState>>,
}

impl VMTrait for LVM {
    fn initialize(&mut self, args: &[&dyn Any]) -> Result<(), Error> {
        let account_store_path = args[0].downcast_ref::<String>().expect("Failed to downcast account store path");
        let contract_store_path = args[1].downcast_ref::<String>().expect("Failed to downcast contract store path");
        let transaction_store_path = args[2].downcast_ref::<String>().expect("Failed to downcast transaction store path");
        self.state = Some(Arc::new(LVMState::new(account_store_path, contract_store_path, transaction_store_path)?));
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

    pub fn create_enviroment(&self) -> Result<(), Error> {
        self.lua.sandbox(false)?;

        let globals = self.lua.globals();
        let env = self.lua.create_table()?;

        for pair in globals.pairs::<String, Value>() {
            if let Ok((key, value)) = pair {
                match key.as_str() {
                    "_contract" | "_sender" => continue,
                    _ => ()
                }

                //todo!("deep-copy value maybe")
                env.raw_set(key, value)?;
            }
        }

        self.lua.sandbox(true).map_err(|e| Error::from(e))
    }

    pub fn get_enviroment(&self, env_key: &String) -> Option<LuaTable> {
        match self.lua.named_registry_value(&env_key).ok() {
            Some(table) => Some(table),
            None => {
                self.create_enviroment().ok()?;
                self.lua.named_registry_value(&env_key).ok()
            }
        }
    }

    pub fn load_bytecode(&self, bytecode: &[u8]) -> Result<Function, Error> {
        self.lua.load(bytecode).into_function().map_err(|e| Error::from(e))
    }
}

pub struct Contract {
    pub address: Option<Address>,
    pub code: Option<Vec<u8>>,
    pub lvm_state: Arc<LVMState>,
    pub chunk: Option<Function>,
}

impl ContractTrait for Contract {
    fn from_code(data: &[&dyn Any]) -> Result<Self, Error> {
        let address = data[0].downcast_ref::<Address>().unwrap();
        let code = data[1].downcast_ref::<&[u8]>().unwrap();
        let lvm_state = data[2].downcast_ref::<Arc<LVMState>>().unwrap();

        let env = lvm_state.lua.get_enviroment(&get_env_key(address)).expect("Failed to get enviroment");

        let chunk = lvm_state.lua.load_bytecode(code);
        if let Err(e) = chunk {
            return Err(e);
        }

        chunk.as_ref().expect("Failed to load bytecode").set_environment(env)?;
        let chunk = chunk.unwrap();
        chunk.call::<bool>(())?; 

        Ok(Self { 
            address: Some(address.clone()),
            code: Some(code.to_vec()),
            lvm_state: lvm_state.clone(),
            chunk: Some(chunk),
        })
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

    fn perform_upgrade(&self, _from: &Address, _code: &[u8]) -> Result<(), Error> {
        /*match self.call_function(from, &Self::hash_function_name("can_upgrade"), &[&ContractValueType::Address(from.clone())]) {
            Ok(ContractValueType::Bool(true)) => Ok(()),
            _ => Err(Error::msg("Failed to upgrade contract, no permission")),
        }*/
        unimplemented!("perform_upgrade not implemented")
    }

    fn call_function(&self, _from: &Address, function: &String, args: &[&ContractValueType]) -> Result<ContractValueType, Error> {
        let lua = &self.lvm_state.lua.lua;
        let env = self.lvm_state.lua.get_enviroment(&self.get_env_key()).expect("Failed to get enviroment");

        let function = env.raw_get::<Function>(lua.create_string(&function.as_bytes())?)?;
        let lua_args: Result<Vec<_>, _> = args.iter().map(|arg| (*arg).clone().into_lua(lua)).collect();
        match function.call::<ContractValueType>(lua_args?) {
            Ok(b) => Ok(b),
            Err(e) => Err(Error::from(e)),
        }
    }

    fn get_from_storage(&self, _key: &[u8]) -> Result<Vec<u8>, Error> {
        unimplemented!("get_from_storage not implemented")
    }

    fn put_to_storage(&self, _key: &[u8], _value: &[u8]) -> Result<(), Error> {
        unimplemented!("put_to_storage not implemented")
    }
}

impl Contract {
    pub fn get_env_key(&self) -> String {
        get_env_key(self.address.as_ref().unwrap())
    }
}

pub fn get_env_key(address: &Address) -> String {
    format!("env_{}", address)
}