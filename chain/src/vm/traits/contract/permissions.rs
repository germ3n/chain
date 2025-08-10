use {
    crate::vm::ContractTrait,
    crate::structs::account::Address,
    crate::vm::contract::ContractValueType,
    crate::crypto::hash::{Hash, HashAlgorithm},
};

pub trait Permissions: ContractTrait {
    fn can_call(&self, from: &Address, function: Hash) -> bool {
        match self.call_function(from, Hash::from_bytes(HashAlgorithm::Blake3_256, b"can_call"), &[&ContractValueType::Hash(function)]) {
            Ok(ContractValueType::Bool(true)) => true,
            _ => false,
        }
    }
}