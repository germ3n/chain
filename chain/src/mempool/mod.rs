use {
    crate::structs::tx::Tx,
    crate::crypto::hash::HashAlgorithm,
    crate::crypto::hash::Hashable,
    std::collections::HashMap,
    std::sync::RwLock,
    crate::crypto::hash::Hash,
    anyhow::Error,
    std::sync::Arc,
    std::collections::{HashSet, BTreeSet},
    std::sync::atomic::AtomicUsize,
    std::sync::atomic::Ordering,
    cutedogs::document,
};

pub struct Mempool {
    pub txs: RwLock<HashMap<Hash, Arc<Tx>>>,
    pub txs_by_fee: RwLock<BTreeSet<MempoolEntry>>,
    pub tx_count: AtomicUsize,
}

#[derive(Eq, PartialEq, Clone)]
pub struct MempoolEntry {
    fee: u64,
    tx: Arc<Tx>,
    tx_hash: Hash,
}

impl Ord for MempoolEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.fee.cmp(&self.fee)
             .then_with(|| self.tx_hash.cmp(&other.tx_hash))
    }
}

impl PartialOrd for MempoolEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Mempool {
    #[document(summary = "Creates a new mempool", returns = "The new mempool")]
    pub fn new() -> Self {
        Self { 
            txs: RwLock::new(HashMap::new()),
            txs_by_fee: RwLock::new(BTreeSet::new()),
            tx_count: AtomicUsize::new(0),
        }
    }

    #[document(summary = "Adds a transaction to the mempool", params = {
        "tx": "The transaction to add"
    }, returns = "The result of the addition")]
    pub fn add_tx(&self, tx: Tx) -> Result<(), Error> {
        match self.txs.write().unwrap().insert(tx.hash(HashAlgorithm::Blake3_256), Arc::new(tx)) {
            Some(_) => Err(anyhow::Error::msg("Transaction already exists in mempool")),
            None => {
                self.tx_count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            },
        }
    }

    #[document(summary = "Removes a transaction from the mempool", params = {
        "hash": "The hash of the transaction to remove"
    }, returns = "The result of the removal")]
    pub fn remove_tx(&self, hash: Hash) -> Result<(), Error> {
        match self.txs.write().unwrap().remove(&hash) {
            Some(_) => {
                self.tx_count.fetch_sub(1, Ordering::Relaxed);
                Ok(())
            },
            None => Err(anyhow::Error::msg("Transaction not found in mempool")),
        }
    }

    #[document(summary = "Gets a transaction from the mempool", params = {
        "hash": "The hash of the transaction to get"
    }, returns = "The transaction")]
    pub fn get_tx(&self, hash: Hash) -> Result<Arc<Tx>, Error> {
        match self.txs.read().unwrap().get(&hash) {
            Some(tx) => Ok(tx.clone()),
            None => Err(anyhow::Error::msg("Transaction not found in mempool")),
        }
    }

    #[document(summary = "Removes transactions from the mempool", params = {
        "hashes": "The hashes of the transactions to remove"
    })]
    pub fn remove_txs(&self, hashes: HashSet<Hash>) {
        self.txs.write().unwrap().retain(|hash, _| {
            if hashes.contains(hash) {
                self.tx_count.fetch_sub(1, Ordering::Relaxed);
                false
            } else {
                true
            }
        });
    }

    #[document(summary = "Gets the number of transactions in the mempool", returns = "The number of transactions in the mempool")]
    pub fn tx_count(&self) -> usize {
        self.tx_count.load(Ordering::Relaxed)
    }

    #[document(summary = "Gets the transactions in the mempool in batches", returns = "The transactions in the mempool in batches")]
    pub fn get_tx_batches(&self) -> Vec<Vec<Arc<Tx>>> {
        self.txs.read().unwrap().values().cloned().collect::<Vec<_>>().chunks(100).map(|chunk| chunk.to_vec()).collect()
    }

    #[document(summary = "Gets the transactions in the mempool sorted by fee", returns = "The transactions in the mempool sorted by fee")]
    pub fn get_txs_by_fee(&self) -> Vec<Arc<Tx>> {
        self.txs_by_fee.read().unwrap().iter().map(|entry| entry.tx.clone()).collect()
    }
}

impl std::fmt::Display for Mempool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mempool {{ tx_count: {}, txs: {{ {} }} }}", 
            self.tx_count.load(Ordering::Relaxed),
            self.txs.read().unwrap().iter().map(|(hash, tx)| format!("0x{}: 0x{}", hash, tx.hash(HashAlgorithm::Blake3_256))).collect::<Vec<_>>().join(", ")
        )
    }
}