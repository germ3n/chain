use {
    bincode::{Encode, Decode},
    crate::crypto::hash::{HashAlgorithm, Hash},
    crate::structs::tx::{Tx, TxData},
    crate::crypto::hash::Hashable,
    anyhow::Error,
};

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct Block {
    pub hash: Hash,
    pub prev_hash: Hash,
    pub nonce: u64,
    pub timestamp: u64,
    pub txs: Vec<Tx>,
}

impl Block {
    pub fn genesis() -> Self {
        let mut block = Self {
            hash: Hash::null(),
            prev_hash: Hash::null(),
            nonce: 0,
            timestamp: 0,
            txs: vec![],
        };
        block.hash = block.hash(HashAlgorithm::Blake3_256);
        block
    }

    pub fn coinbase(prev_hash: Hash) -> Self {
        let mut block = Self {
            hash: Hash::null(),
            prev_hash: prev_hash,
            nonce: 0,
            timestamp: 0,
            txs: vec![Tx::coinbase()],
        };
        block.hash = block.hash(HashAlgorithm::Blake3_256);
        block
    }

    pub fn ensure_first_tx_is_coinbase(&self) -> Result<(), Error> {
        if !matches!(self.txs[0].data, TxData::Coinbase(_)) {
            return Err(Error::msg("First tx is not coinbase"));
        }
        Ok(())
    }

    pub fn ensure_single_coinbase(&self) -> Result<(), Error> {
        for tx in self.txs.iter().skip(1) {
            if matches!(tx.data, TxData::Coinbase(_)) {
                return Err(Error::msg("Block has more than one coinbase tx"));
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.ensure_first_tx_is_coinbase()?;
        self.ensure_single_coinbase()?;
        if self.hash != self.hash(HashAlgorithm::Blake3_256) {
            return Err(Error::msg("Block hash does not match calculated hash"));
        }
        Ok(())
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "Block {{ hash: 0x{}, prev_hash: 0x{}, nonce: {}, timestamp: {}, txs: {{ {} }} }}", 
            self.hash, 
            self.prev_hash, 
            self.nonce, 
            self.timestamp, 
            self.txs.iter().map(|tx| tx.hash(HashAlgorithm::Blake3_256)).collect::<Vec<_>>().into_iter().map(|hash| format!("0x{}", hash)).collect::<Vec<_>>().join(", ")
        )
    }
}