use {
    crate::crypto::hash::Hash,
    crate::structs::{account::Address, tx::Tx},
    crate::vm::contract::ContractTrait,
    anyhow::Error,
    std::any::Any,
    bincode::{Encode, Decode},
    serde::{Serialize, Deserialize},
    crate::generate_enum_with_type_id
};

pub mod contract;
pub mod traits;

#[cfg(feature = "evm")]
pub mod evm;
#[cfg(feature = "lvm")]
pub mod lvm;
#[cfg(feature = "svm")]
pub mod svm;
#[cfg(feature = "pvm")]
pub mod pvm;

generate_enum_with_type_id! {
    u8,
    
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
    #[repr(u8)]
    pub enum VMType {
        #[cfg(feature = "evm")]
        EVM = 0,
        #[cfg(feature = "lvm")]
        LVM = 1,
        //#[cfg(feature = "svm")]
        //SVM = 2,
        //#[cfg(feature = "pvm")]
        //PVM = 3,
    }
}

pub trait VMTrait {
    fn initialize(&mut self, args: &[&dyn Any]) -> Result<(), Error>;
    fn prepare_execution(&self) -> Result<(), Error>;
    fn execute_tx(&self, tx: &Tx) -> Result<(), Error>;
    fn finalize_execution(&self) -> Result<(), Error>;
    fn start_tx(&self) -> Result<(), Error>;
    fn commit_tx(&self) -> Result<(), Error>;
    fn rollback_tx(&self) -> Result<(), Error>;
    fn deploy_contract(&self, code: &[u8]) -> Result<Address, Error>;
    fn get_contract(&self, hash: &Hash) -> Result<Option<&dyn ContractTrait>, Error>;
    fn get_state(&self) -> Result<&dyn Any, Error>;
    fn get_vm_type(&self) -> VMType;
}