use {
    crate::{
        crypto::{hash::Hash, keypair::{Signature, PublicKey}}, 
        structs::{account::Address, num::U256},
    }, 
    anyhow::Error, 
    std::any::Any, 
    bincode::{Encode, Decode},
    crate::crypto::hash::{Hashable, HashAlgorithm},
};

#[derive(Debug, Clone, PartialEq, Encode, Decode, Eq, Hash)]
#[repr(u8)]
pub enum ContractValueType {
    U8(u8) = 0,
    I8(i8) = 1,
    U16(u16) = 2,
    I16(i16) = 3,
    U32(u32) = 4,
    I32(i32) = 5,
    U64(u64) = 6,
    I64(i64) = 7,
    U128(u128) = 8,
    I128(i128) = 9,
    U256(U256) = 10,
    String(String) = 11,
    Bytes(Vec<u8>) = 12,
    Hash(Hash) = 13,
    Address(Address) = 14,
    Signature(Signature) = 15,
    PublicKey(PublicKey) = 16,
    Bool(bool) = 17,
}

pub trait ContractTrait: Any {
    fn from_code(data: &[&dyn Any]) -> Result<Self, Error> where Self: Sized;
    fn get_code(&self) -> Option<&[u8]>;
    fn perform_upgrade(&self, from: &Address, code: &[u8]) -> Result<(), Error>;
    fn get_address(&self) -> Option<&Address>;
    fn call_function(&self, from: &Address, function: &String, args: &[&ContractValueType]) -> Result<ContractValueType, Error>;
    fn get_from_storage(&self, key: &[u8]) -> Result<Vec<u8>, Error>;
    fn put_to_storage(&self, key: &[u8], value: &[u8]) -> Result<(), Error>;

    fn hash_function_name(function: &str) -> String // needs _ prefix
    where Self: Sized {
        format!("_0x{}", function.as_bytes().hash(HashAlgorithm::XXHash3_64))
    }
}