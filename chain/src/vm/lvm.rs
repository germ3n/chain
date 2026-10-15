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
    mlua::{Function, Value, Table, AnyUserData, prelude::LuaString},
    crate::crypto::hash::{Hashable, HashAlgorithm},
    cutedogs::document,
    rayon::prelude::*,
    crate::serialize::binary::SerializableBytes,
    std::collections::HashMap,
    std::sync::RwLock,
    std::sync::Mutex,
};

pub mod num;
pub mod compiler;
pub mod analyzer;
pub mod currency;
pub mod types;

pub struct LVM {
    pub state: Option<Arc<LVMState>>,
}

impl LVM {
    pub fn new() -> Self {
        Self {
            state: None,
        }
    }

    pub fn execute_transactions(&self, txs: &[&Tx]) -> Result<(), Error> {
        let state = self.state.as_ref().ok_or_else(|| Error::msg("LVM not initialized"))?.clone();
        txs.iter().try_for_each(|tx| -> Result<(), Error> {
            let contract_address = Address::Null;
            let contract_code = state.contracts.read().unwrap().get(&contract_address).ok_or(Error::msg("Contract code not found"))?.clone();

            let contract = Contract::from_code(&[
                &Some(contract_address.clone()) as &dyn Any,
                &contract_code as &dyn Any,
                &state as &dyn Any,
            ])?;

            let function_name = "some_function_from_tx"; // todo: implement
            let args: Vec<&ContractValueType> = Vec::new();

            contract.call_function(&Address::Null, function_name, &args)?;
            Ok(())
        })?;
        Ok(())
    }
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
        Ok(())
    }

    #[document(summary = "Executes a transaction on the LVM", params = {
        "tx": "The transaction to execute"
    }, returns = "The result of the execution")]
    fn execute_tx(&self, tx: &Tx) -> Result<(), Error> {
        self.execute_transactions(&[tx])
    }

    #[document(summary = "Finalizes the execution of the LVM", returns = "The result of the finalization")]
    fn finalize_execution(&self) -> Result<(), Error> {
        Ok(())
    }

    #[document(summary = "Starts a transaction on the LVM", returns = "The result of the start")]
    fn start_tx(&self) -> Result<(), Error> {
        Ok(())
    }

    #[document(summary = "Commits a transaction on the LVM", returns = "The result of the commit")]
    fn commit_tx(&self) -> Result<(), Error> {
        Ok(())
    }

    #[document(summary = "Rolls back a transaction on the LVM", returns = "The result of the rollback")]
    fn rollback_tx(&self) -> Result<(), Error> {
        Ok(())
    }

    #[document(summary = "Deploys a contract on the LVM", params = {
        "code": "The code to deploy"
    }, returns = "The address of the deployed contract")]
    fn deploy_contract(&self, _code: &[u8]) -> Result<Address, Error> {
        Ok(Address::Null)
    }

    #[document(summary = "Gets a contract from the LVM", params = {
        "hash": "The hash of the contract"
    }, returns = "The contract")]
    fn get_contract(&self, _hash: &Hash) -> Result<Option<&dyn ContractTrait>, Error> {
        Ok(None)
    }

    #[document(summary = "Gets the state of the LVM", returns = "The state of the LVM")]
    fn get_state(&self) -> Result<&dyn Any, Error> {
        Ok(self.state.as_ref().unwrap())
    }

    #[document(summary = "Gets the type of the LVM", returns = "The type of the LVM")]
    fn get_vm_type(&self) -> VMType {
        VMType::LVM
    }
}

pub struct LVMState {
    pub accounts: Arc<Box<dyn DatabaseTrait>>,
    pub contracts: RwLock<HashMap<Address, Vec<u8>>>,
    pub transactions: Arc<Box<dyn DatabaseTrait>>,
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
            contracts: RwLock::new(HashMap::new()),
            transactions: open_db(transaction_store_path.to_owned())?,
        };
        info!("LVM state initialized");
        Ok(state)
    }
}

fn prepare_lua_state(lua: &Lua, _sender: &Address, _contract_address: &Address) -> Result<Table, Error> {
    lua.sandbox(true)?;
    let globals = lua.globals();
    let env = lua.create_table()?;
    num::register_all_numeric_libraries(lua)?;

    let deep_copy_value = |lua: &Lua, value: Value| -> Result<Value, Error> {
        fn deep_copy_recursive(lua: &Lua, value: Value) -> Result<Value, mlua::Error> {
            match value {
                Value::Table(table) => {
                    let new_table = lua.create_table()?;
                    for pair in table.pairs::<Value, Value>() {
                        let (key, val) = pair?;
                        let copied_key = deep_copy_recursive(lua, key)?;
                        let copied_val = deep_copy_recursive(lua, val)?;
                        new_table.raw_set(copied_key, copied_val)?;
                    }
                    Ok(Value::Table(new_table))
                }
                _ => Ok(value)
            }
        }
        deep_copy_recursive(lua, value).map_err(Error::from)
    };

    for pair in globals.pairs::<String, Value>() {
        if let Ok((key, value)) = pair {
            let copied_value = deep_copy_value(lua, value)?;
            env.raw_set(key, copied_value)?;
        }
    }
    Ok(env)
}


pub struct Contract {
    pub address: Option<Address>,
    pub code: Option<Vec<u8>>,
    pub lvm_state: Arc<LVMState>,
    lua_state: Arc<Mutex<Option<(Lua, Table)>>>,
}

impl ContractTrait for Contract {
    #[document(summary = "Creates a new contract from code", params = {
        "data": "The data to create the contract from"
    }, returns = "The new contract")]
    fn from_code(data: &[&dyn Any]) -> Result<Self, Error> {
        let address = data[0].downcast_ref::<Option<Address>>().ok_or_else(|| Error::msg("Invalid address"))?.clone();
        let code = data[1].downcast_ref::<&[u8]>().ok_or_else(|| Error::msg("Invalid code"))?.to_vec();
        let lvm_state = data[2].downcast_ref::<Arc<LVMState>>().ok_or_else(|| Error::msg("Invalid LVM state"))?.clone();
        Ok(Self {
            address,
            code: Some(code),
            lvm_state,
            lua_state: Arc::new(Mutex::new(None)),
        })
    }

    #[document(summary = "Gets the code of the contract", returns = "The code of the contract")]
    fn get_code(&self) -> Option<&[u8]> {
        self.code.as_deref()
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
        unimplemented!("perform_upgrade not implemented")
    }
    
    #[document(summary = "Gets a function from the contract", params = {
        "function": "The function to get"
    }, returns = "The function")]
    fn get_function(&self, function: &str) -> Result<Arc<dyn Any>, Error> {
        let (lua, env) = self.get_or_create_lua_state()?;
        let func: Function = env.get(function)?;
        Ok(Arc::new(func))
    }

    #[document(summary = "Calls a function on the contract", params = {
        "from": "The address of the caller",
        "function": "The function to call",
        "args": "The arguments to the function"
    }, returns = "The result of the function call")]
    fn call_function(&self, from: &Address, function: &str, args: &[&ContractValueType]) -> Result<ContractValueType, Error> {
        let (lua, env) = self.get_or_create_lua_state()?;
        let original_from: Option<LuaString> = env.raw_get("_sender")?; // backup the original sender
        env.raw_set("_sender", from.to_string())?; // set the new sender
        let func: Function = env.get(function)?;
        let lua_args: Result<Vec<_>, _> = args.iter().map(|arg| (*arg).clone().into_lua(&lua)).collect();
        let result: ContractValueType = func.call(lua_args?)?;
        env.raw_set("_sender", original_from)?; // restore the original sender
        Ok(result)
    }
}

impl Contract {
    #[document(summary = "Gets the Lua state of the contract", returns = "The Lua state of the contract")]
    fn get_or_create_lua_state(&self) -> Result<(Lua, Table), Error> {
        let mut state_guard = self.lua_state.lock().unwrap();
        
        if let Some((ref lua, ref env)) = *state_guard {
            return Ok((lua.clone(), env.clone()));
        }

        let contract_address = self.get_address().ok_or_else(|| Error::msg("Contract has no address"))?;
        let code = self.get_code().ok_or_else(|| Error::msg("Contract has no code"))?;
        
        let lua = Lua::new();
        let env = prepare_lua_state(&lua, &Address::Null, contract_address)?;
        let chunk = lua.load(code).into_function()?;
        chunk.set_environment(env.clone())?;
        chunk.call::<()>(())?;
        
        let lua_clone = lua.clone();
        let env_clone = env.clone();
        *state_guard = Some((lua, env));
        
        Ok((lua_clone, env_clone))
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