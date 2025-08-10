use {
    crate::{
        crypto::{hash::{Hash, HashAlgorithm, Hashable}, keypair::{PublicKey, Signature}}, 
        structs::{account::Address, num::U256},
    }, anyhow::Error, bincode::{Decode, Encode},
    mlua::{prelude::*, Error as LuaError, FromLua, IntoLua, Value}, 
    std::any::Any
};

#[derive(Debug, Clone, PartialEq, Encode, Decode, Eq, Hash)]
#[repr(u8)]
pub enum ContractValueType {
    None = 0,
    U8(u8) = 1,
    I8(i8) = 2,
    U16(u16) = 3,
    I16(i16) = 4,
    U32(u32) = 5,
    I32(i32) = 6,
    U64(u64) = 7,
    I64(i64) = 8,
    U128(u128) = 9,
    I128(i128) = 10,
    U256(U256) = 11,
    String(String) = 12,
    Bytes(Vec<u8>) = 13,
    Hash(Hash) = 14,
    Address(Address) = 15,
    Signature(Signature) = 16,
    PublicKey(PublicKey) = 17,
    Bool(bool) = 18,
}

pub trait ContractTrait: Any {
    fn from_code(data: &[&dyn Any]) -> Result<Self, Error> where Self: Sized;
    fn get_code(&self) -> Option<&[u8]>;
    fn perform_upgrade(&self, from: &Address, code: &[u8]) -> Result<(), Error>;
    fn get_address(&self) -> Option<&Address>;
    fn call_function(&self, from: &Address, function: &str, args: &[&ContractValueType]) -> Result<ContractValueType, Error>;
    fn get_from_storage(&self, key: &[u8]) -> Result<Vec<u8>, Error>;
    fn put_to_storage(&self, key: &[u8], value: &[u8]) -> Result<(), Error>;

    fn hash_function_name(function: &str) -> String // needs _ prefix
    where Self: Sized {
        format!("_0x{}", function.as_bytes().hash(HashAlgorithm::XXHash3_64))
    }
}

pub struct ContractAnalysisOptions {
    pub max_call_depth: u32, // max number of calls before getting put into last batch, to prevent DoS
}

pub trait ContractAnalysis {
    fn interferes_with(&self, other: &dyn ContractAnalysis, options: &ContractAnalysisOptions) -> Result<bool, Error>;
}

pub trait ContractAnalyzerTrait: ContractTrait {
    fn analyze(&self) -> Result<(), Error>;
    fn get_analysis(&self) -> Result<&dyn ContractAnalysis, Error>;
    fn get_analysis_options(&self) -> &ContractAnalysisOptions;
}

impl IntoLua for ContractValueType {
    fn into_lua(self, lua: &Lua) -> Result<Value, LuaError> {
        match self {
            ContractValueType::None => Ok(Value::Nil),
            ContractValueType::U8(value) => Ok(Value::Integer(value.into())),
            ContractValueType::I8(value) => Ok(Value::Integer(value.into())),
            ContractValueType::U16(value) => Ok(Value::Integer(value.into())),
            ContractValueType::I16(value) => Ok(Value::Integer(value.into())),
            ContractValueType::I32(value) => Ok(Value::Integer(value.into())),
            /*ContractValueType::U32(value) => Ok(Value::Integer(value.into())),
            ContractValueType::U64(value) => Ok(Value::Integer(value.into())),
            ContractValueType::I64(value) => Ok(Value::Integer(value.into())),
            ContractValueType::U128(value) => Ok(Value::Integer(value.into())),
            ContractValueType::I128(value) => Ok(Value::Integer(value.into())),
            ContractValueType::U256(value) => Ok(Value::Integer(value.into())),*/
            ContractValueType::String(value) => Ok(Value::String(lua.create_string(&value.as_bytes())?)),
            //ContractValueType::Bytes(value) => Ok(Value::Binary(value)),
            //ContractValueType::Hash(value) => Ok(Value::String(lua.create_string(&value.as_bytes()).map_err(LuaError::from)?)),
            //ContractValueType::Address(value) => Ok(Value::String(lua.create_string(&value.as_bytes()).map_err(LuaError::from)?)),
            //ContractValueType::Signature(value) => Ok(Value::String(lua.create_string(&value.as_bytes()).map_err(LuaError::from)?)),
            //ContractValueType::PublicKey(value) => Ok(Value::String(lua.create_string(&value.as_bytes()).map_err(LuaError::from)?)),
            ContractValueType::Bool(value) => Ok(Value::Boolean(value)),
            _ => Err(LuaError::RuntimeError("Invalid contract value type".to_string())),
        }
    }
}

impl FromLua for ContractValueType {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self, LuaError> {
        match value {
            Value::Nil => Ok(ContractValueType::None),
            Value::Integer(value) => {
                if value < 0 {
                    match value {
                        -128..=127 => Ok(ContractValueType::I8(value as i8)),
                        -32768..=32767 => Ok(ContractValueType::I16(value as i16)),
                        -2147483648..=2147483647 => Ok(ContractValueType::I32(value as i32)),
                    }
                } else {
                    match value as u32 {
                        0..=255 => Ok(ContractValueType::U8(value as u8)),
                        0..=65535 => Ok(ContractValueType::U16(value as u16)),
                        0..=4294967295 => Ok(ContractValueType::U32(value as u32)),
                    }
                }
            },
            Value::String(value) => Ok(ContractValueType::String(value.to_str()?.to_string())),
            Value::Boolean(value) => Ok(ContractValueType::Bool(value)),
            _ => Err(LuaError::RuntimeError("Invalid contract value type".to_string())),
        }
    }
}