use {
    crate::structs::tx::Tx,
    anyhow::Error,
    crate::crypto::hash::Hash,
    std::any::Any,
    crate::vm::contract::ContractTrait,
    crate::db::DatabaseTrait,
};

pub mod contract;
#[cfg(feature = "evm")]
pub mod evm;
#[cfg(feature = "lvm")]
pub mod lvm;

pub trait VMTrait<D: DatabaseTrait> {
    fn init(&self) -> Result<(), Error>;
    fn prepare_execution(&self) -> Result<(), Error>;
    fn execute_tx(&self, tx: &Tx) -> Result<(), Error>;
    fn finalize_execution(&self) -> Result<(), Error>;
    fn start_tx(&self) -> Result<(), Error>;
    fn commit_tx(&self) -> Result<(), Error>;
    fn rollback_tx(&self) -> Result<(), Error>;
    fn deploy_contract(&self, code: &[u8]) -> Result<Hash, Error>;
    fn get_contract(&self, hash: &Hash) -> Result<Option<&dyn ContractTrait<D>>, Error>;
    fn get_state(&self) -> Result<&dyn Any, Error>;
}