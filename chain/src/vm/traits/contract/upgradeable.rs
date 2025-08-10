use {
    crate::{
        vm::ContractTrait,
        structs::account::Address,
        vm::traits::contract::permissions::Permissions,
    },
    anyhow::Error,
};

pub trait Upgradeable: ContractTrait + Permissions
where Self: Sized {
    fn upgrade(&self, from: &Address, code: &[u8]) -> Result<(), Error> {
        match self.has_permission(from, "upgrade") {
            Ok(true) => self.perform_upgrade(from, code),
            Ok(false) => Err(Error::msg("Failed to upgrade contract, no permission")),
            Err(e) => Err(e),
        }
    }
}