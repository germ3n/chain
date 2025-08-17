use {
    crate::{
        crypto::hash::{Hash, HashAlgorithm},
        structs::account::Address,
    },
    hex,
    mlua::{prelude::*, AnyUserData, UserDataMethods, UserData},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UHash(pub Hash);

impl UserData for UHash {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, _: ()| {
            Ok(this.0.to_string())
        });

        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, other: AnyUserData| {
            if let Ok(other) = other.borrow::<Self>() {
                Ok(this.0 == other.0)
            } else {
                Ok(false)
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UAddress(pub Address);

impl UserData for UAddress {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, _: ()| {
            Ok(this.0.to_string())
        });

        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, other: AnyUserData| {
            if let Ok(other) = other.borrow::<Self>() {
                Ok(this.0 == other.0)
            } else {
                Ok(false)
            }
        });
    }
}

pub fn create_hash_lib(lua: &Lua) -> LuaResult<LuaTable> {
    let globals = lua.globals();
    globals.set("XXHASH3_64", HashAlgorithm::XXHash3_64 as u8)?;
    #[cfg(feature = "blake3")]
    globals.set("BLAKE3_256", HashAlgorithm::Blake3_256 as u8)?;
    #[cfg(feature = "keccak")]
    globals.set("KECCAK256", HashAlgorithm::Keccak256 as u8)?;
    let lib = lua.create_table()?;
    let hash_func = lua.create_function(|_, (algo, bytes): (u8, LuaString)| {
        if let Ok(algorithm) = HashAlgorithm::try_from(algo) {
            let hash = Hash::from_bytes(algorithm, &bytes.as_bytes());
            Ok(UHash(hash))
        } else {
            Err(LuaError::RuntimeError("Unsupported hash algorithm".to_string()))
        }
    })?;
    lib.set("bytes", &hash_func)?;
    lib.set("string", &hash_func)?;

    let metatable = lua.create_table()?;
    metatable.set(
        "__call",
        lua.create_function(|_, (_, algo, bytes): (LuaTable, u8, LuaString)| {
            if let Ok(algorithm) = HashAlgorithm::try_from(algo) {
                let hash = Hash::from_bytes(algorithm, &bytes.as_bytes());
                Ok(UHash(hash))
            } else {
                Err(LuaError::RuntimeError("Unsupported hash algorithm".to_string()))
            }
        })?,
    )?;
    lib.set_metatable(Some(metatable));
    Ok(lib)
}

pub fn create_address_lib(lua: &Lua) -> LuaResult<LuaTable> {
    let globals = lua.globals();
    globals.set("ED25519", Address::Ed25519([0; 32]).algorithm_id())?;
    globals.set("EDBLAKE", Address::EdBlake([0; 32]).algorithm_id())?;
    let lib = lua.create_table()?;
    lib.set("null", lua.create_function(|_, ()| {
        Ok(UAddress(Address::null()))
    })?)?;
    let from_hex_func = lua.create_function(|_, (s, addr_type): (String, Option<u8>)| {
        let s = if s.starts_with("0x") { &s[2..] } else { &s };
        let bytes = hex::decode(s).map_err(LuaError::external)?;
        if bytes.len() != 32 {
            return Err(LuaError::external("Invalid address length"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        
        let address = match addr_type.unwrap_or(Address::Ed25519([0; 32]).algorithm_id()) {
            0 => Address::Null,
            1 => Address::Ed25519(arr),
            2 => Address::EdBlake(arr),
            _ => return Err(LuaError::external("Unsupported address type")),
        };

        Ok(UAddress(address))
    })?;
    lib.set("from_hex", &from_hex_func)?;

    let metatable = lua.create_table()?;
    metatable.set(
        "__call",
        lua.create_function(|_, (_, s, addr_type): (LuaTable, String, Option<u8>)| {
            let s = if s.starts_with("0x") { &s[2..] } else { &s };
            let bytes = hex::decode(s).map_err(LuaError::external)?;
            if bytes.len() != 32 {
                return Err(LuaError::external("Invalid address length"));
            }
            let address = match addr_type.unwrap_or(Address::Ed25519([0; 32]).algorithm_id()) {
                1 => Address::Ed25519(bytes.try_into().map_err(|_| LuaError::external("Invalid address length"))?),
                2 => Address::EdBlake(bytes.try_into().map_err(|_| LuaError::external("Invalid address length"))?),
                _ => return Err(LuaError::external("Unsupported address type")),
            };

            Ok(UAddress(address))
        })?,
    )?;
    lib.set_metatable(Some(metatable));
    Ok(lib)
}