use {
    bincode::{Encode, Decode},
    crate::crypto::hash::{HashAlgorithm, Hash},
    crate::structs::tx::{Tx, TxData},
    crate::crypto::hash::{Hashable, HashableBytes},
    anyhow::Error,
    serde::{Serialize, Deserialize},
    crate::serialize::binary::to_bytes,
    std::borrow::Cow,
    cutedogs::document,
};

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize)]
pub struct BlockHeader {
    pub hash: Hash,
    pub prev_hash: Hash,
    pub nonce: u64,
    pub timestamp: u64,
    pub config_checksum: Hash,
}

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<Tx>,
}

impl Block {
    #[document(summary = "Create the genesis block", returns = "The genesis block")]
    pub fn genesis(hash_algorithm: Option<HashAlgorithm>) -> Self {
        let mut block = Self {
            header: BlockHeader {
                hash: Hash::null(),
                prev_hash: Hash::null(),
                nonce: 0,
                timestamp: 0,
                config_checksum: Hash::null(),
            },
            txs: vec![],
        };
        block.header.hash = block.hash(hash_algorithm.unwrap_or(HashAlgorithm::Blake3_256));
        block.header.config_checksum = block.hash(HashAlgorithm::XXHash3_64);
        block
    }

    #[document(summary = "Creates a coinbase block", params = {
        "prev_hash": "The previous block hash"
    }, returns = "The coinbase block")]
    pub fn coinbase(prev_hash: Hash, hash_algorithm: Option<HashAlgorithm>) -> Self {
        let mut block = Self {
            header: BlockHeader {
                hash: Hash::null(),
                prev_hash: prev_hash,
                nonce: 0,
                timestamp: 0,
                config_checksum: Hash::null(),
            },
            txs: vec![Tx::coinbase()],
        };
        block.header.hash = block.hash(hash_algorithm.unwrap_or(HashAlgorithm::Blake3_256));
        block.header.config_checksum = block.hash(HashAlgorithm::XXHash3_64);
        block
    }

    #[document(summary = "Ensures the first transaction is a coinbase transaction", returns = "The result of the validation")]
    pub fn ensure_first_tx_is_coinbase(&self) -> Result<(), Error> {
        if !matches!(self.txs[0].data, TxData::Coinbase(_)) {
            return Err(Error::msg("First tx is not coinbase"));
        }
        Ok(())
    }

    #[document(summary = "Ensures the block has only one coinbase transaction", returns = "The result of the validation")]
    pub fn ensure_single_coinbase(&self) -> Result<(), Error> {
        for tx in self.txs.iter().skip(1) {
            if matches!(tx.data, TxData::Coinbase(_)) {
                return Err(Error::msg("Block has more than one coinbase tx"));
            }
        }
        Ok(())
    }

    #[document(summary = "Validates a block", returns = "The result of the validation")]
    pub fn validate(&self, hash_algorithm: Option<HashAlgorithm>) -> Result<(), Error> {
        self.ensure_first_tx_is_coinbase()?;
        self.ensure_single_coinbase()?;
        if self.header.hash != self.hash(hash_algorithm.unwrap_or(HashAlgorithm::Blake3_256)) {
            return Err(Error::msg("Block hash does not match calculated hash"));
        }
        Ok(())
    }
}

impl HashableBytes for Block {
    fn get_hashable_bytes(&self) -> Result<Cow<[u8]>, Error> {
        let bytes = to_bytes(&(
            &self.header.prev_hash,
            &self.header.nonce,
            &self.header.timestamp,
            &self.txs,
        ))?;
        Ok(Cow::Owned(bytes))
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "Block {{ hash: 0x{}, prev_hash: 0x{}, nonce: {}, timestamp: {}, config_checksum: 0x{}, txs: {{ {} }} }}", 
            self.header.hash, 
            self.header.prev_hash, 
            self.header.nonce, 
            self.header.timestamp, 
            self.header.config_checksum,
            self.txs.iter().map(|tx| tx.hash(HashAlgorithm::Blake3_256)).collect::<Vec<_>>().into_iter().map(|hash| format!("0x{}", hash)).collect::<Vec<_>>().join(", ") // todo: use specified hash algorithm
        )
    }
}