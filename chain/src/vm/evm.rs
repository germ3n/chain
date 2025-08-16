use {
    crate::vm::{VMTrait, VMType},
    crate::structs::tx::Tx,
    anyhow::Error,
    crate::crypto::hash::Hash,
    std::any::Any,
    crate::db::DatabaseTrait,
    std::sync::Arc,
    crate::vm::contract::{ContractTrait, ContractValueType, VMStorageTrait},
    crate::structs::account::Address,
    crate::db::open_db,
    cutedogs::document,
};

pub struct EVM {
    pub state: Option<EVMState>,
}

impl VMTrait for EVM {
    #[document(summary = "Initializes the EVM", params = {
        "args": "The arguments to initialize the EVM"
    }, returns = "The result of the initialization")]
    fn initialize(&mut self, args: &[&dyn Any]) -> Result<(), Error> {
        let account_store_path = args[0].downcast_ref::<String>().expect("Invalid account store path");
        let contract_store_path = args[1].downcast_ref::<String>().expect("Invalid contract store path");
        let transaction_store_path = args[2].downcast_ref::<String>().expect("Invalid transaction store path");
        self.state = Some(EVMState::new(account_store_path, contract_store_path, transaction_store_path)?);
        Ok(())
    }

    #[document(summary = "Prepares the execution of the EVM", returns = "The result of the preparation")]
    fn prepare_execution(&self) -> Result<(), Error> {
        Ok(())
    }

    #[document(summary = "Executes a transaction on the EVM", params = {
        "tx": "The transaction to execute"
    }, returns = "The result of the execution")]
    fn execute_tx(&self, _tx: &Tx) -> Result<(), Error> {
        Ok(())
    }

    #[document(summary = "Finalizes the execution of the EVM", returns = "The result of the finalization")]
    fn finalize_execution(&self) -> Result<(), Error> {
        Ok(())
    }

    #[document(summary = "Starts a transaction on the EVM", returns = "The result of the start")]
    fn start_tx(&self) -> Result<(), Error> {
        Ok(())
    }

    #[document(summary = "Commits a transaction on the EVM", returns = "The result of the commit")]
    fn commit_tx(&self) -> Result<(), Error> {
        Ok(())
    }

    #[document(summary = "Rolls back a transaction on the EVM", returns = "The result of the rollback")]
    fn rollback_tx(&self) -> Result<(), Error> {
        Ok(())
    }

    #[document(summary = "Deploys a contract on the EVM", params = {
        "code": "The code to deploy"
    }, returns = "The address of the deployed contract")]
    fn deploy_contract(&self, _code: &[u8]) -> Result<Address, Error> {
        Ok(Address::null())
    }

    #[document(summary = "Gets a contract from the EVM", params = {
        "hash": "The hash of the contract"
    }, returns = "The contract")]
    fn get_contract(&self, _hash: &Hash) -> Result<Option<&dyn ContractTrait>, Error> {
        Ok(None)
    }

    #[document(summary = "Gets the state of the EVM", returns = "The state of the EVM")]
    fn get_state(&self) -> Result<&dyn Any, Error> {
        match &self.state {
            Some(state) => Ok(state as &dyn Any),
            None => Err(Error::msg("EVM state not initialized"))
        }
    }

    #[document(summary = "Gets the type of the EVM", returns = "The type of the EVM")]
    fn get_vm_type(&self) -> VMType {
        VMType::EVM
    }
}

impl EVM {
    pub fn new() -> Self {
        Self { 
            state: None,
        }
    }
}

pub struct EVMState {
    pub accounts: Arc<Box<dyn DatabaseTrait>>,
    pub contracts: Arc<Box<dyn DatabaseTrait>>,
    pub transactions: Arc<Box<dyn DatabaseTrait>>,
}

impl EVMState {
    #[document(summary = "Creates a new EVM state", params = {
        "account_store_path": "The path to the account store",
        "contract_store_path": "The path to the contract store",
        "transaction_store_path": "The path to the transaction store"
    }, returns = "The new EVM state")]
    pub fn new(account_store_path: &String, contract_store_path: &String, transaction_store_path: &String) -> Result<Self, Error> {
        Ok(Self { 
            accounts: open_db(account_store_path.to_owned())?, 
            contracts: open_db(contract_store_path.to_owned())?, 
            transactions: open_db(transaction_store_path.to_owned())?,
        })
    }
}

pub struct Contract {
    pub storage: Arc<Box<dyn DatabaseTrait>>,
    pub address: Option<Address>,
    pub code: Option<Vec<u8>>,
}

impl Contract {
    #[document(summary = "Creates a new contract from code", params = {
        "storage": "The storage of the contract",
        "address": "The address of the contract",
        "code": "The code of the contract"
    }, returns = "The new contract")]
    pub fn from_code(storage: Arc<Box<dyn DatabaseTrait>>, address: Option<Address>, code: Vec<u8>) -> Self {
        Self { 
            storage, 
            address, 
            code: Some(code)
        }
    }
}

impl ContractTrait for Contract {
    #[document(summary = "Creates a new contract from code", params = {
        "data": "The data to create the contract from"
    }, returns = "The new contract")]
    fn from_code(data: &[&dyn Any]) -> Result<Self, Error> {
        let storage = data[0].downcast_ref::<Arc<Box<dyn DatabaseTrait>>>().expect("Invalid database");
        let address = data[1].downcast_ref::<Address>();
        let code = data[2].downcast_ref::<&[u8]>().expect("Invalid code");
        Ok(Self { 
            storage: storage.clone(), 
            address: address.cloned(), 
            code: Some(code.to_vec()) 
        })
    }

    #[document(summary = "Gets the code of the contract", returns = "The code of the contract")]
    fn get_code(&self) -> Option<&[u8]> {
        match self.code {
            Some(ref code) => Some(code),
            None => None,
        }
    }

    #[document(summary = "Performs an upgrade on the contract", params = {
        "from": "The address of the contract to upgrade",
        "code": "The code to upgrade the contract to"
    }, returns = "The result of the upgrade")]
    fn perform_upgrade(&self, _from: &Address, _code: &[u8]) -> Result<(), Error> {
        unimplemented!("perform_upgrade not implemented")
    }

    #[document(summary = "Gets the address of the contract", returns = "The address of the contract")]
    fn get_address(&self) -> Option<&Address> {
        self.address.as_ref()
    }

    #[document(summary = "Gets a function from the contract", params = {
        "function": "The function to get"
    }, returns = "The function")]
    fn get_function(&self, _function: &str) -> Result<Arc<dyn Any>, Error> {
        unimplemented!("get_function not implemented")
    }

    #[document(summary = "Calls a function on the contract", params = {
        "from": "The address of the caller",
        "function": "The function to call",
        "args": "The arguments to the function"
    }, returns = "The result of the function call")]
    fn call_function(&self, _from: &Address, _function: &str, _args: &[&ContractValueType]) -> Result<ContractValueType, Error> {
        unimplemented!("call_function not implemented")
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