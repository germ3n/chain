use {
    crate::{
        vm::ContractTrait,
        structs::account::Address,
        crypto::hash::{Hash, HashAlgorithm},
        vm::traits::contract::permissions::Permissions,
    },
    anyhow::Error,
};

pub trait Upgradeable: ContractTrait + Permissions {
    fn upgrade(&self, from: &Address, code: &[u8]) -> Result<(), Error> {
        match self.can_call(from, Hash::from_bytes(HashAlgorithm::Blake3_256, b"can_upgrade")) {
            true => self.execute_upgrade(from, code),
            false => Err(Error::msg("Failed to upgrade contract, no permission")),
        }
    }
    
    fn execute_upgrade(&self, from: &Address, code: &[u8]) -> Result<(), Error>;
}