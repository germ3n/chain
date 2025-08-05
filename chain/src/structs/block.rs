use {
    bincode::{Encode, Decode},
    crate::crypto::hash::{HashAlgorithm, Hash},
    crate::structs::tx::{Tx, TxData},
    crate::crypto::keypair::PublicKey,
    crate::crypto::hash::Hashable,
    anyhow::Error,
};

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct Block {
    pub hash: Hash,
    pub prev_hash: Hash,
    pub nonce: u64,
    pub miner_pubkey: PublicKey,
    pub timestamp: u64,
    pub txs: Vec<Tx>,
}

impl Block {
    pub fn genesis() -> Self {
        let mut block = Self {
            hash: Hash::null(),
            prev_hash: Hash::null(),
            nonce: 0,
            miner_pubkey: PublicKey::null(),
            timestamp: 0,
            txs: vec![],
        };
        block.hash = block.hash(HashAlgorithm::Blake3_256);
        block
    }

    pub fn coinbase(prev_hash: Hash, miner_pubkey: PublicKey) -> Self {
        let mut block = Self {
            hash: Hash::null(),
            prev_hash: prev_hash,
            nonce: 0,
            miner_pubkey: miner_pubkey,
            timestamp: 0,
            txs: vec![Tx::coinbase()],
        };
        block.hash = block.hash(HashAlgorithm::Blake3_256);
        block
    }

    pub fn ensure_first_tx_is_coinbase(&mut self) -> Result<(), Error> {
        if !matches!(self.txs[0].data, TxData::Coinbase(_)) {
            return Err(Error::msg("First tx is not coinbase"));
        }
        Ok(())
    }

    pub fn ensure_single_coinbase(&mut self) -> Result<(), Error> {
        for tx in self.txs.iter().skip(1) {
            if matches!(tx.data, TxData::Coinbase(_)) {
                return Err(Error::msg("Block has more than one coinbase tx"));
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "Block {{ hash: {}, prev_hash: {}, nonce: {}, miner_pubkey: {}, timestamp: {}, txs: {:?} }}", 
            self.hash, 
            self.prev_hash, 
            self.nonce, 
            self.miner_pubkey, 
            self.timestamp, 
            self.txs.iter().map(|tx| tx.hash(HashAlgorithm::Blake3_256)).collect::<Vec<_>>()
        )
    }
}