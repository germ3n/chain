use {
    crate::vm::VMTrait,
    crate::structs::tx::Tx,
    anyhow::Error,
    crate::crypto::hash::Hash,
    std::any::Any,
    crate::db::DatabaseTrait,
    std::sync::Arc,
};

pub struct EVM<D: DatabaseTrait> {
    pub state: Option<Arc<EVMState<D>>>,
}

impl<D: DatabaseTrait + 'static> VMTrait for EVM<D> {
    fn init(&self) -> Result<(), Error> {
        Ok(())
    }

    fn prepare_execution(&self) -> Result<(), Error> {
        //Err(Error::msg("Not implemented"))
        Ok(())
    }

    fn execute_tx(&self, _tx: &Tx) -> Result<(), Error> {
        //Err(Error::msg("Not implemented"))
        Ok(())
    }

    fn finalize_execution(&self) -> Result<(), Error> {
        //Err(Error::msg("Not implemented"))
        Ok(())
    }

    fn start_tx(&self) -> Result<(), Error> {
        //Err(Error::msg("Not implemented"))
        Ok(())
    }

    fn commit_tx(&self) -> Result<(), Error> {
        //Err(Error::msg("Not implemented"))
        Ok(())
    }

    fn rollback_tx(&self) -> Result<(), Error> {
        //Err(Error::msg("Not implemented"))
        Ok(())
    }

    fn deploy_contract(&self, _code: &[u8]) -> Result<Hash, Error> {
        //Err(Error::msg("Not implemented"))
        Ok(Hash::null())
    }

    fn get_contract(&self, _hash: &Hash) -> Result<Option<&dyn Any>, Error> {
        //Err(Error::msg("Not implemented"))
        Ok(None)
    }

    fn get_state(&self) -> Result<&dyn Any, Error> {
        match &self.state {
            Some(state) => Ok(state.as_ref() as &dyn Any),
            None => Err(Error::msg("EVM state not initialized"))
        }
    }
}

impl<D: DatabaseTrait + 'static> EVM<D> {
    pub fn new(db: Arc<D>) -> Self {
        Self { state: Some(Arc::new(EVMState::new(db))) }
    }
}

pub struct EVMState<D: DatabaseTrait> {
    pub accounts: Arc<D>,
    pub contracts: Arc<D>,
    pub transactions: Arc<D>,
}

impl<D: DatabaseTrait> EVMState<D> {
    pub fn new(db: Arc<D>) -> Self {
        Self { accounts: db.clone(), contracts: db.clone(), transactions: db.clone() }
    }
}