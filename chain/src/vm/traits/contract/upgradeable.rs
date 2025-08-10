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
        match self.can_call(from, &String::from("upgrade")) {
            true => self.perform_upgrade(from, code),
            false => Err(Error::msg("Failed to upgrade contract, no permission")),
        }
    }
}