use {
    crate::vm::ContractTrait,
    crate::structs::account::Address,
    crate::vm::contract::ContractValueType,
};

pub trait Permissions: ContractTrait
where Self: Sized {
    fn can_call(&self, from: &Address, function: &String) -> bool
    where Self: Sized {
        match self.call_function(from, &Self::hash_function_name("can_call"), &[&ContractValueType::Address(from.clone()), &ContractValueType::String(function.clone())]) {
            Ok(ContractValueType::Bool(true)) => true,
            _ => false,
        }
    }
}