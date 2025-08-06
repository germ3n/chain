use {
    crate::db::DatabaseTrait,
    crate::structs::account::Address,
    std::any::Any,
    std::sync::Arc,
    anyhow::Error,
};

pub trait ContractTrait<D: DatabaseTrait>: Any + Send + Sync {
    fn new(data: &dyn Any) -> Self where Self: Sized;
    fn get_code(&self) -> Result<Arc<Vec<u8>>, Error>;
    fn get_address(&self) -> &Address;
    fn get_last_modified(&self) -> Option<u64>;
}