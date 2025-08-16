use {
    crate::{
        vm::ContractTrait,
        structs::account::Address,
        vm::traits::contract::permissions::Permissions,
    },
    anyhow::Error,
    cutedogs::document,
};

pub trait Upgradeable: ContractTrait + Permissions
where Self: Sized {
    #[document(summary = "Upgrades a contract", params = {
        "from": "The address of the contract to upgrade",
        "code": "The code to upgrade the contract to"
    }, returns = "The result of the upgrade")]
    fn upgrade(&self, from: &Address, code: &[u8]) -> Result<(), Error> {
        match self.has_permission(from, "upgrade") {
            Ok(true) => self.perform_upgrade(from, code),
            Ok(false) => Err(Error::msg("Failed to upgrade contract, no permission")),
            Err(e) => Err(e),
        }
    }
}