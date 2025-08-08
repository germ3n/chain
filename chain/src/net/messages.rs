use {
    crate::crypto::hash::Hash,
    crate::structs::tx::Tx,
    crate::structs::block::Block,
    bincode::{Encode, Decode},
};

#[derive(Debug, PartialEq, Encode, Decode)]
#[repr(u16)]
pub enum Message {
    Ping = 0,
    Pong = 1,
    GetHeight = 2,
    Height((Hash, u64)) = 3, // (block hash, block height)
    GetBlock(Hash) = 4,
    Block(Block) = 5,
    SubmitTransaction(Tx) = 6,
    TransactionSubmitted(Hash) = 7,
    TransactionRejected(Hash) = 8,
    BroadcastBlock(Block) = 9,
    BroadcastTransaction(Tx) = 10,
}