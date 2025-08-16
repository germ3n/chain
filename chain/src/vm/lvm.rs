use {
    crate::vm::{VMTrait, contract::{ContractTrait, ContractValueType, VMStorageTrait}, VMType},
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
    std::collections::HashMap,
    std::sync::RwLock,
    crate::crypto::hash::{Hashable, HashAlgorithm},
    cutedogs::document,
};


pub mod num;
pub mod compiler;
pub mod analyzer;
pub mod currency;

pub struct LVM {
    pub state: Option<Arc<LVMState>>,
}

impl VMTrait for LVM {
    #[document(summary = "Initializes the LVM", params = {
        "args": "The arguments to initialize the LVM"
    }, returns = "The result of the initialization")]
    fn initialize(&mut self, args: &[&dyn Any]) -> Result<(), Error> {
        let account_store_path = args[0].downcast_ref::<String>().expect("Failed to downcast account store path");
        let contract_store_path = args[1].downcast_ref::<String>().expect("Failed to downcast contract store path");
        let transaction_store_path = args[2].downcast_ref::<String>().expect("Failed to downcast transaction store path");
        self.state = Some(Arc::new(LVMState::new(account_store_path, contract_store_path, transaction_store_path)?));
        Ok(())
    }

    #[document(summary = "Prepares the execution of the LVM", returns = "The result of the preparation")]
    fn prepare_execution(&self) -> Result<(), Error> {
        //unimplemented!("prepare_execution not implemented")
        Ok(())
    }

    #[document(summary = "Executes a transaction on the LVM", params = {
        "tx": "The transaction to execute"
    }, returns = "The result of the execution")]
    fn execute_tx(&self, _tx: &Tx) -> Result<(), Error> {
        //unimplemented!("execute_tx not implemented")
        Ok(())
    }

    #[document(summary = "Finalizes the execution of the LVM", returns = "The result of the finalization")]
    fn finalize_execution(&self) -> Result<(), Error> {
        //unimplemented!("finalize_execution not implemented")
        Ok(())
    }

    #[document(summary = "Starts a transaction on the LVM", returns = "The result of the start")]
    fn start_tx(&self) -> Result<(), Error> {
        //unimplemented!("start_tx not implemented")
        Ok(())
    }

    #[document(summary = "Commits a transaction on the LVM", returns = "The result of the commit")]
    fn commit_tx(&self) -> Result<(), Error> {
        //unimplemented!("commit_tx not implemented")
        Ok(())
    }

    #[document(summary = "Rolls back a transaction on the LVM", returns = "The result of the rollback")]
    fn rollback_tx(&self) -> Result<(), Error> {
        //unimplemented!("rollback_tx not implemented")
        Ok(())
    }

    #[document(summary = "Deploys a contract on the LVM", params = {
        "code": "The code to deploy"
    }, returns = "The address of the deployed contract")]
    fn deploy_contract(&self, _code: &[u8]) -> Result<Address, Error> {
        //unimplemented!("deploy_contract not implemented")
        Ok(Address::Null)
    }

    #[document(summary = "Gets a contract from the LVM", params = {
        "hash": "The hash of the contract"
    }, returns = "The contract")]
    fn get_contract(&self, _hash: &Hash) -> Result<Option<&dyn ContractTrait>, Error> {
        //unimplemented!("get_contract not implemented")
        Ok(None)
    }

    #[document(summary = "Gets the state of the LVM", returns = "The state of the LVM")]
    fn get_state(&self) -> Result<&dyn Any, Error> {
        //unimplemented!("get_state not implemented")
        Ok(self.state.as_ref().unwrap())
    }

    #[document(summary = "Gets the type of the LVM", returns = "The type of the LVM")]
    fn get_vm_type(&self) -> VMType {
        VMType::LVM
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
    #[document(summary = "Creates a new LVM state", params = {
        "account_store_path": "The path to the account store",
        "contract_store_path": "The path to the contract store",
        "transaction_store_path": "The path to the transaction store"
    }, returns = "The new LVM state")]
    pub fn new(account_store_path: &String, contract_store_path: &String, transaction_store_path: &String) -> Result<Self, Error> {
        let state = Self { 
            accounts: open_db(account_store_path.to_owned())?, 
            contracts: open_db(contract_store_path.to_owned())?, 
            transactions: open_db(transaction_store_path.to_owned())?,
            lua: LuaEnv::new()
        };
        
        num::register_all_numeric_libraries(&state.lua.lua)?;

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

    #[document(summary = "Creates a new enviroment", returns = "The new enviroment")]
    pub fn create_enviroment(&self) -> Result<LuaTable, Error> {
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

        self.lua.sandbox(true).map_err(|e| Error::from(e))?;

        Ok(env)
    }

    #[document(summary = "Gets an enviroment", params = {
        "env_key": "The key of the enviroment"
    }, returns = "The enviroment")]
    pub fn get_enviroment(&self, env_key: Option<&str>) -> Option<LuaTable> {
        match env_key {
            Some(key) => {
                match self.lua.named_registry_value(key).ok() {
                    Some(table) => Some(table),
                    None => self.create_enviroment().ok()
                }
            }
            None => self.create_enviroment().ok()
        }
    }

    #[document(summary = "Loads a bytecode", params = {
        "bytecode": "The bytecode to load"
    }, returns = "The loaded bytecode")]
    pub fn load_bytecode(&self, bytecode: &[u8]) -> Result<Function, Error> {
        self.lua.load(bytecode).into_function().map_err(|e| Error::from(e))
    }
}

pub struct Contract {
    pub address: Option<Address>,
    pub code: Option<Vec<u8>>,
    pub lvm_state: Arc<LVMState>,
    pub chunk: Option<Function>,
    pub env: Option<LuaTable>,
    pub functions: RwLock<HashMap<Hash, Arc<Function>>>,
}

impl ContractTrait for Contract {
    #[document(summary = "Creates a new contract from code", params = {
        "data": "The data to create the contract from"
    }, returns = "The new contract")]
    fn from_code(data: &[&dyn Any]) -> Result<Self, Error> {
        let address = data[0].downcast_ref::<Option<Address>>().unwrap();
        let code = data[1].downcast_ref::<&[u8]>().unwrap();
        let lvm_state = data[2].downcast_ref::<Arc<LVMState>>().unwrap();

        let env = match address {
            Some(address) => lvm_state.lua.get_enviroment(Some(&get_env_key(address))).expect("Failed to get enviroment"),
            None => lvm_state.lua.create_enviroment().expect("Failed to create enviroment"),
        };

        let chunk = lvm_state.lua.load_bytecode(code);
        if let Err(e) = chunk {
            return Err(e);
        }

        chunk.as_ref().expect("Failed to load bytecode").set_environment(env.clone())?;
        let chunk = chunk.unwrap();
        chunk.call::<bool>(())?;
        
        let mut functions = HashMap::new();
        for pair in env.pairs::<String, Function>() {
            if let Ok((key, value)) = pair {
                functions.insert(key.hash(HashAlgorithm::XXHash3_64), Arc::new(value));
            }
        }

        Ok(Self { 
            address: address.clone(),
            code: Some(code.to_vec()),
            lvm_state: lvm_state.clone(),
            chunk: Some(chunk),
            env: Some(env),
            functions: RwLock::new(functions),
        })
    }

    #[document(summary = "Gets the code of the contract", returns = "The code of the contract")]
    fn get_code(&self) -> Option<&[u8]> {
        match self.code {
            Some(ref code) => Some(code),
            None => None,
        }
    }

    #[document(summary = "Gets the address of the contract", returns = "The address of the contract")]
    fn get_address(&self) -> Option<&Address> {
        self.address.as_ref()
    }

    #[document(summary = "Performs an upgrade on the contract", params = {
        "from": "The address of the contract to upgrade",
        "code": "The code to upgrade the contract to"
    }, returns = "The result of the upgrade")]
    fn perform_upgrade(&self, _from: &Address, _code: &[u8]) -> Result<(), Error> {
        /*match self.call_function(from, &Self::hash_function_name("can_upgrade"), &[&ContractValueType::Address(from.clone())]) {
            Ok(ContractValueType::Bool(true)) => Ok(()),
            _ => Err(Error::msg("Failed to upgrade contract, no permission")),
        }*/
        unimplemented!("perform_upgrade not implemented")
    }

    #[document(summary = "Gets a function from the contract", params = {
        "function": "The function to get"
    }, returns = "The function")]
    fn get_function(&self, function: &str) -> Result<Arc<dyn Any>, Error> {
        let function = self.functions.read().unwrap().get(&function.hash(HashAlgorithm::XXHash3_64)).ok_or(Error::msg("Function not found"))?.clone();
        Ok(function)
    }

    #[document(summary = "Calls a function on the contract", params = {
        "from": "The address of the caller",
        "function": "The function to call",
        "args": "The arguments to the function"
    }, returns = "The result of the function call")]
    fn call_function(&self, _from: &Address, function: &str, args: &[&ContractValueType]) -> Result<ContractValueType, Error> {
        let lua_args: Result<Vec<_>, _> = args.iter().map(|arg| (*arg).clone().into_lua(&self.lvm_state.lua.lua)).collect();
        self.get_function(function)?.downcast_ref::<Function>().unwrap().call::<ContractValueType>(lua_args?).map_err(Error::from)
    }
}

impl VMStorageTrait for Contract {
    #[document(summary = "Gets a value from the storage of the contract", params = {
        "key": "The key to get"
    }, returns = "The value from the storage")]
    fn get_from_storage(&self, _key: &[u8]) -> Result<Vec<u8>, Error> {
        unimplemented!("get_from_storage not implemented")
    }

    #[document(summary = "Puts a value to the storage of the contract", params = {
        "key": "The key to put",
        "value": "The value to put"
    }, returns = "The result of the put")]
    fn put_to_storage(&self, _key: &[u8], _value: &[u8]) -> Result<(), Error> {
        unimplemented!("put_to_storage not implemented")
    }
}

impl Contract {
    #[document(summary = "Gets the enviroment key of the contract", returns = "The enviroment key of the contract")]
    pub fn get_env_key(&self) -> Option<String> {
        match self.address {
            Some(ref address) => Some(get_env_key(address)),
            None => None,
        }
    }
}

#[document(summary = "Gets the enviroment key of the contract", returns = "The enviroment key of the contract")]
pub fn get_env_key(address: &Address) -> String {
    format!("env_{}", address)
}