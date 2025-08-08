use {
    crate::{
        crypto::{hash::Hash, keypair::{Signature, PublicKey}}, 
        structs::account::Address,
    }, 
    anyhow::Error, 
    std::any::Any, 
    bincode::{Encode, Decode},
};

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
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
    String(String) = 10,
    Bytes(Vec<u8>) = 11,
    Hash(Hash) = 12,
    Address(Address) = 13,
    Signature(Signature) = 14,
    PublicKey(PublicKey) = 15,
    Bool(bool) = 16,
}

pub trait ContractTrait: Any + Send {
    fn new(data: &[&dyn Any]) -> Self where Self: Sized;
    fn get_code(&self) -> Option<&[u8]>;
    fn upgrade(&self, code: Vec<u8>) -> Result<(), Error>;
    fn get_address(&self) -> Option<&Address>;
    fn call_function(&self, function: Hash, args: &[&ContractValueType]) -> Result<Vec<u8>, Error>;
    fn get_from_storage(&self, key: &[u8]) -> Result<Vec<u8>, Error>;
    fn put_to_storage(&self, key: &[u8], value: &[u8]) -> Result<(), Error>;
}