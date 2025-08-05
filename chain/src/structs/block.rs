use {
    bincode::{Encode, Decode},
    crate::crypto::hash::{HashAlgorithm, Hash},
    crate::structs::tx::Tx,
    crate::crypto::keypair::PublicKey,
    crate::crypto::hash::Hashable,
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

    pub fn coinbase_block(prev_hash: Hash, miner_pubkey: PublicKey) -> Self {
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
}