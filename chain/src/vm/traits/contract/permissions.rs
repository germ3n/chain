use {
    crate::vm::ContractTrait,
    crate::structs::account::Address,
    crate::vm::contract::ContractValueType,
    anyhow::Error,
    cutedogs::document,
};

pub trait Permissions: ContractTrait
where Self: Sized {
    #[document(summary = "Checks if a function has a permission", params = {
        "from": "The address of the function caller",
        "function": "The function to check the permission of"
    }, returns = "The result of the permission check")]
    fn has_permission(&self, from: &Address, function: &str) -> Result<bool, Error>
    where Self: Sized {
        match self.call_function(from, &Self::hash_function_name("has_permission"), &[&ContractValueType::Address(from.clone()), &ContractValueType::String(function.to_string())]) {
            Ok(ContractValueType::Bool(true)) => Ok(true),
            Ok(ContractValueType::Bool(false)) => Ok(false),
            Err(e) => Err(e),
            _ => Err(Error::msg("Failed to get permission")),
        }
    }
}