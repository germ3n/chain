use {
    crate::vm::VMTrait,
    crate::structs::tx::Tx,
    anyhow::Error,
    crate::crypto::hash::Hash,
    std::any::Any,
    std::sync::Arc,
    crate::db::DatabaseTrait,
};

pub struct LVM<D: DatabaseTrait> {
    pub state: Option<LVMState<D>>,
}

impl<D: DatabaseTrait> VMTrait for LVM<D> {
    fn init(&self) -> Result<(), Error> {
        Ok(())
    }

    fn prepare_execution(&self) -> Result<(), Error> {
        Err(Error::msg("Not implemented"))
    }

    fn execute_tx(&self, _tx: &Tx) -> Result<(), Error> {
        Err(Error::msg("Not implemented"))
    }

    fn finalize_execution(&self) -> Result<(), Error> {
        Err(Error::msg("Not implemented"))
    }

    fn start_tx(&self) -> Result<(), Error> {
        Err(Error::msg("Not implemented"))
    }

    fn commit_tx(&self) -> Result<(), Error> {
        Err(Error::msg("Not implemented"))
    }

    fn rollback_tx(&self) -> Result<(), Error> {
        Err(Error::msg("Not implemented"))
    }

    fn deploy_contract(&self, _code: &[u8]) -> Result<Hash, Error> {
        Err(Error::msg("Not implemented"))
    }

    fn get_contract(&self, _hash: &Hash) -> Result<Option<&dyn Any>, Error> {
        Err(Error::msg("Not implemented"))
    }

    fn get_state(&self) -> Result<&dyn Any, Error> {
        Err(Error::msg("Not implemented"))
    }
}

impl<D: DatabaseTrait> LVM<D> {
    pub fn new(db: Arc<D>) -> Self {
        Self { state: Some(LVMState::new(db)) }
    }
}

pub struct LVMState<D: DatabaseTrait> {
    pub accounts: Arc<D>,
    pub contracts: Arc<D>,
    pub transactions: Arc<D>,
}

impl<D: DatabaseTrait> LVMState<D> {
    pub fn new(db: Arc<D>) -> Self {
        Self { accounts: db.clone(), contracts: db.clone(), transactions: db.clone() }
    }
}