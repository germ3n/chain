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
};

pub struct EVM {
    pub state: Option<EVMState>,
}

impl VMTrait for EVM {
    fn initialize(&mut self, args: &[&dyn Any]) -> Result<(), Error> {
        let account_store_path = args[0].downcast_ref::<String>().expect("Invalid account store path");
        let contract_store_path = args[1].downcast_ref::<String>().expect("Invalid contract store path");
        let transaction_store_path = args[2].downcast_ref::<String>().expect("Invalid transaction store path");
        self.state = Some(EVMState::new(account_store_path, contract_store_path, transaction_store_path)?);
        Ok(())
    }

    fn prepare_execution(&self) -> Result<(), Error> {
        Ok(())
    }

    fn execute_tx(&self, _tx: &Tx) -> Result<(), Error> {
        Ok(())
    }

    fn finalize_execution(&self) -> Result<(), Error> {
        Ok(())
    }

    fn start_tx(&self) -> Result<(), Error> {
        Ok(())
    }

    fn commit_tx(&self) -> Result<(), Error> {
        Ok(())
    }

    fn rollback_tx(&self) -> Result<(), Error> {
        Ok(())
    }

    fn deploy_contract(&self, _code: &[u8]) -> Result<Address, Error> {
        Ok(Address::null())
    }

    fn get_contract(&self, _hash: &Hash) -> Result<Option<&dyn ContractTrait>, Error> {
        Ok(None)
    }

    fn get_state(&self) -> Result<&dyn Any, Error> {
        match &self.state {
            Some(state) => Ok(state as &dyn Any),
            None => Err(Error::msg("EVM state not initialized"))
        }
    }

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
    pub fn from_code(storage: Arc<Box<dyn DatabaseTrait>>, address: Option<Address>, code: Vec<u8>) -> Self {
        Self { 
            storage, 
            address, 
            code: Some(code)
        }
    }
}

impl ContractTrait for Contract {
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

    fn get_code(&self) -> Option<&[u8]> {
        match self.code {
            Some(ref code) => Some(code),
            None => None,
        }
    }

    fn perform_upgrade(&self, _from: &Address, _code: &[u8]) -> Result<(), Error> {
        unimplemented!("perform_upgrade not implemented")
    }

    fn get_address(&self) -> Option<&Address> {
        self.address.as_ref()
    }

    fn get_function(&self, _function: &str) -> Result<Arc<dyn Any>, Error> {
        unimplemented!("get_function not implemented")
    }

    fn call_function(&self, _from: &Address, _function: &str, _args: &[&ContractValueType]) -> Result<ContractValueType, Error> {
        unimplemented!("call_function not implemented")
    }
}

impl VMStorageTrait for Contract {
    fn get_from_storage(&self, _key: &[u8]) -> Result<Vec<u8>, Error> {
        unimplemented!("get_from_storage not implemented")
    }

    fn put_to_storage(&self, _key: &[u8], _value: &[u8]) -> Result<(), Error> {
        unimplemented!("put_to_storage not implemented")
    }
}