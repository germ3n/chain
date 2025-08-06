use {
    crate::structs::tx::Tx,
    crate::crypto::hash::HashAlgorithm,
    crate::crypto::hash::Hashable,
    std::collections::HashMap,
    std::sync::RwLock,
    crate::crypto::hash::Hash,
    anyhow::Error,
    std::sync::Arc,
    std::collections::HashSet,
};

pub struct Mempool {
    pub txs: RwLock<HashMap<Hash, Arc<Tx>>>,
}

impl Mempool {
    pub fn new() -> Self {
        Self { txs: RwLock::new(HashMap::new()) }
    }

    pub fn add_tx(&mut self, tx: Tx) -> Result<(), Error> {
        match self.txs.write().unwrap().insert(tx.hash(HashAlgorithm::Blake3_256), Arc::new(tx)) {
            Some(_) => Err(anyhow::Error::msg("Transaction already exists in mempool")),
            None => Ok(()),
        }
    }

    pub fn remove_tx(&mut self, hash: Hash) -> Result<(), Error> {
        match self.txs.write().unwrap().remove(&hash) {
            Some(_) => Ok(()),
            None => Err(anyhow::Error::msg("Transaction not found in mempool")),
        }
    }

    pub fn get_tx(&self, hash: Hash) -> Result<Arc<Tx>, Error> {
        match self.txs.read().unwrap().get(&hash) {
            Some(tx) => Ok(tx.clone()),
            None => Err(anyhow::Error::msg("Transaction not found in mempool")),
        }
    }

    pub fn for_each_tx(&self, callback: impl Fn(&Hash, &Tx)) {
        self.txs.read().unwrap().iter().for_each(|(hash, tx)| callback(hash, tx));
    }

    pub fn remove_txs(&self, hashes: HashSet<Hash>) {
        self.txs.write().unwrap().retain(|hash, _| !hashes.contains(hash));
    }
}

impl std::fmt::Display for Mempool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mempool {{ txs: {{ {} }} }}", 
            self.txs.read().unwrap().iter().map(|(hash, tx)| format!("0x{}: 0x{}", hash, tx.hash(HashAlgorithm::Blake3_256))).collect::<Vec<_>>().join(", ")
        )
    }
}