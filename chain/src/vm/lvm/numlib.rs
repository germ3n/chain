use {
    mlua::{Lua, UserData, UserDataMethods, Result as LuaResult, MetaMethod, Value, FromLua, IntoLua},
    std::{fmt, mem, cmp::Ordering},
    bincode::{Decode, Encode},
    serde::{Serialize, Deserialize},
    crate::structs::num::U256,
    primitive_types::U256 as PrimitiveU256,
    log::info,
};

trait NumericValue {
    fn as_u128(&self) -> Option<u128>;
    fn as_i128(&self) -> Option<i128>;
    fn as_u256(&self) -> U256;
    fn is_signed(&self) -> bool;
    fn size_bytes(&self) -> usize;
    fn type_name(&self) -> &'static str;
}

trait CrossCompare {
    fn cross_cmp(&self, other: &dyn NumericValue) -> Option<Ordering>;
    fn cross_eq(&self, other: &dyn NumericValue) -> bool;
}

#[derive(Debug, Clone, Copy)]
enum NumericResult {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256(LuaU256),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
}

impl NumericResult {
    fn smart_add(a: &dyn NumericValue, b: &dyn NumericValue) -> Self {
        let a_size = a.size_bytes();
        let b_size = b.size_bytes();
        let a_signed = a.is_signed();
        let b_signed = b.is_signed();
        
        if a_size <= 8 && b_size <= 8 {
            if let (Some(a_val), Some(b_val)) = (a.as_u128(), b.as_u128()) {
                let result = a_val.saturating_add(b_val);
                
                if !a_signed && !b_signed {
                    if result <= u8::MAX as u128 { return NumericResult::U8(result as u8); }
                    if result <= u16::MAX as u128 { return NumericResult::U16(result as u16); }
                    if result <= u32::MAX as u128 { return NumericResult::U32(result as u32); }
                    if result <= u64::MAX as u128 { return NumericResult::U64(result as u64); }
                    return NumericResult::U128(result);
                }
                
                if a_signed || b_signed {
                    if let (Some(a_i), Some(b_i)) = (a.as_i128(), b.as_i128()) {
                        let result = a_i.saturating_add(b_i);
                        if result >= i8::MIN as i128 && result <= i8::MAX as i128 { 
                            return NumericResult::I8(result as i8); 
                        }
                        if result >= i16::MIN as i128 && result <= i16::MAX as i128 { 
                            return NumericResult::I16(result as i16); 
                        }
                        if result >= i32::MIN as i128 && result <= i32::MAX as i128 { 
                            return NumericResult::I32(result as i32); 
                        }
                        if result >= i64::MIN as i128 && result <= i64::MAX as i128 { 
                            return NumericResult::I64(result as i64); 
                        }
                        return NumericResult::I128(result);
                    }
                }
            }
        }
        
        let a_u256 = a.as_u256();
        let b_u256 = b.as_u256();
        let result = a_u256.0 + b_u256.0;
        
        if result <= PrimitiveU256::from(u128::MAX) {
            let val = result.as_u128();
            if !a_signed && !b_signed && val <= u64::MAX as u128 {
                return NumericResult::U64(val as u64);
            }
            return NumericResult::U128(val);
        }
        
        NumericResult::U256(LuaU256(U256(result)))
    }
    
    fn smart_sub(a: &dyn NumericValue, b: &dyn NumericValue) -> Self {
        let a_size = a.size_bytes();
        let b_size = b.size_bytes();
        let a_signed = a.is_signed();
        let b_signed = b.is_signed();
        
        if a_size <= 8 && b_size <= 8 {
            if let (Some(a_val), Some(b_val)) = (a.as_u128(), b.as_u128()) {
                let result = a_val.saturating_sub(b_val);
                if !a_signed && !b_signed {
                    if result <= u8::MAX as u128 { return NumericResult::U8(result as u8); }
                    if result <= u16::MAX as u128 { return NumericResult::U16(result as u16); }
                    if result <= u32::MAX as u128 { return NumericResult::U32(result as u32); }
                    if result <= u64::MAX as u128 { return NumericResult::U64(result as u64); }
                    return NumericResult::U128(result);
                }
            }
            
            if a_signed || b_signed {
                if let (Some(a_i), Some(b_i)) = (a.as_i128(), b.as_i128()) {
                    let result = a_i.saturating_sub(b_i);
                    if result >= i8::MIN as i128 && result <= i8::MAX as i128 { 
                        return NumericResult::I8(result as i8); 
                    }
                    if result >= i16::MIN as i128 && result <= i16::MAX as i128 { 
                        return NumericResult::I16(result as i16); 
                    }
                    if result >= i32::MIN as i128 && result <= i32::MAX as i128 { 
                        return NumericResult::I32(result as i32); 
                    }
                    if result >= i64::MIN as i128 && result <= i64::MAX as i128 { 
                        return NumericResult::I64(result as i64); 
                    }
                    return NumericResult::I128(result);
                }
            }
        }
        
        let a_u256 = a.as_u256();
        let b_u256 = b.as_u256();
        let result = a_u256.0.saturating_sub(b_u256.0);
        
        if result <= PrimitiveU256::from(u128::MAX) {
            let val = result.as_u128();
            if !a_signed && !b_signed && val <= u64::MAX as u128 {
                return NumericResult::U64(val as u64);
            }
            return NumericResult::U128(val);
        }
        
        NumericResult::U256(LuaU256(U256(result)))
    }
    
    fn smart_mul(a: &dyn NumericValue, b: &dyn NumericValue) -> Self {
        let a_size = a.size_bytes();
        let b_size = b.size_bytes();
        
        if a_size <= 4 && b_size <= 4 {
            if let (Some(a_val), Some(b_val)) = (a.as_u128(), b.as_u128()) {
                let result = a_val.saturating_mul(b_val);
                
                if !a.is_signed() && !b.is_signed() {
                    if result <= u8::MAX as u128 { return NumericResult::U8(result as u8); }
                    if result <= u16::MAX as u128 { return NumericResult::U16(result as u16); }
                    if result <= u32::MAX as u128 { return NumericResult::U32(result as u32); }
                    if result <= u64::MAX as u128 { return NumericResult::U64(result as u64); }
                    return NumericResult::U128(result);
                }
            }
        }
        
        if a_size <= 8 && b_size <= 8 {
            if let (Some(a_val), Some(b_val)) = (a.as_u128(), b.as_u128()) {
                let result = a_val.saturating_mul(b_val);
                return NumericResult::U128(result);
            }
        }
        
        let a_u256 = a.as_u256();
        let b_u256 = b.as_u256();
        let result = a_u256.0 * b_u256.0;
        NumericResult::U256(LuaU256(U256(result)))
    }
    
    fn smart_div(a: &dyn NumericValue, b: &dyn NumericValue) -> Self {
        let b_u256 = b.as_u256();
        if b_u256.0.is_zero() {
            return NumericResult::U256(LuaU256(U256(PrimitiveU256::MAX)));
        }
        
        let a_size = a.size_bytes();
        let b_size = b.size_bytes();
        
        if a_size <= 8 && b_size <= 8 {
            if let (Some(a_val), Some(b_val)) = (a.as_u128(), b.as_u128()) {
                if b_val != 0 {
                    let result = a_val / b_val;
                    
                    if !a.is_signed() && !b.is_signed() {
                        if result <= u8::MAX as u128 { return NumericResult::U8(result as u8); }
                        if result <= u16::MAX as u128 { return NumericResult::U16(result as u16); }
                        if result <= u32::MAX as u128 { return NumericResult::U32(result as u32); }
                        if result <= u64::MAX as u128 { return NumericResult::U64(result as u64); }
                        return NumericResult::U128(result);
                    }
                }
            }
        }
        
        let a_u256 = a.as_u256();
        let result = a_u256.0 / b_u256.0;
        
        if result <= PrimitiveU256::from(u128::MAX) {
            let val = result.as_u128();
            if val <= u64::MAX as u128 {
                return NumericResult::U64(val as u64);
            }
            return NumericResult::U128(val);
        }
        
        NumericResult::U256(LuaU256(U256(result)))
    }

    fn smart_bitand(a: &dyn NumericValue, b: &dyn NumericValue) -> Self {
        let a_size = a.size_bytes();
        let b_size = b.size_bytes();
        let a_signed = a.is_signed();
        let b_signed = b.is_signed();
        
        if a_size <= 16 && b_size <= 16 {
            if let (Some(a_val), Some(b_val)) = (a.as_u128(), b.as_u128()) {
                let result = a_val & b_val;
                
                if !a_signed && !b_signed {
                    if result <= u8::MAX as u128 { return NumericResult::U8(result as u8); }
                    if result <= u16::MAX as u128 { return NumericResult::U16(result as u16); }
                    if result <= u32::MAX as u128 { return NumericResult::U32(result as u32); }
                    if result <= u64::MAX as u128 { return NumericResult::U64(result as u64); }
                    return NumericResult::U128(result);
                }
                
                if a_signed || b_signed {
                    if let (Some(a_i), Some(b_i)) = (a.as_i128(), b.as_i128()) {
                        let result = a_i & b_i;
                        if result >= i8::MIN as i128 && result <= i8::MAX as i128 { 
                            return NumericResult::I8(result as i8); 
                        }
                        if result >= i16::MIN as i128 && result <= i16::MAX as i128 { 
                            return NumericResult::I16(result as i16); 
                        }
                        if result >= i32::MIN as i128 && result <= i32::MAX as i128 { 
                            return NumericResult::I32(result as i32); 
                        }
                        if result >= i64::MIN as i128 && result <= i64::MAX as i128 { 
                            return NumericResult::I64(result as i64); 
                        }
                        return NumericResult::I128(result);
                    }
                }
            }
        }
        
        let a_u256 = a.as_u256();
        let b_u256 = b.as_u256();
        let result = a_u256.0 & b_u256.0;
        NumericResult::U256(LuaU256(U256(result)))
    }

    fn smart_bitor(a: &dyn NumericValue, b: &dyn NumericValue) -> Self {
        let a_size = a.size_bytes();
        let b_size = b.size_bytes();
        let a_signed = a.is_signed();
        let b_signed = b.is_signed();
        
        if a_size <= 16 && b_size <= 16 {
            if let (Some(a_val), Some(b_val)) = (a.as_u128(), b.as_u128()) {
                let result = a_val | b_val;
                
                if !a_signed && !b_signed {
                    if result <= u8::MAX as u128 { return NumericResult::U8(result as u8); }
                    if result <= u16::MAX as u128 { return NumericResult::U16(result as u16); }
                    if result <= u32::MAX as u128 { return NumericResult::U32(result as u32); }
                    if result <= u64::MAX as u128 { return NumericResult::U64(result as u64); }
                    return NumericResult::U128(result);
                }
                
                if a_signed || b_signed {
                    if let (Some(a_i), Some(b_i)) = (a.as_i128(), b.as_i128()) {
                        let result = a_i | b_i;
                        if result >= i8::MIN as i128 && result <= i8::MAX as i128 { 
                            return NumericResult::I8(result as i8); 
                        }
                        if result >= i16::MIN as i128 && result <= i16::MAX as i128 { 
                            return NumericResult::I16(result as i16); 
                        }
                        if result >= i32::MIN as i128 && result <= i32::MAX as i128 { 
                            return NumericResult::I32(result as i32); 
                        }
                        if result >= i64::MIN as i128 && result <= i64::MAX as i128 { 
                            return NumericResult::I64(result as i64); 
                        }
                        return NumericResult::I128(result);
                    }
                }
            }
        }
        
        let a_u256 = a.as_u256();
        let b_u256 = b.as_u256();
        let result = a_u256.0 | b_u256.0;
        NumericResult::U256(LuaU256(U256(result)))
    }

    fn smart_bitxor(a: &dyn NumericValue, b: &dyn NumericValue) -> Self {
        let a_size = a.size_bytes();
        let b_size = b.size_bytes();
        let a_signed = a.is_signed();
        let b_signed = b.is_signed();
        
        if a_size <= 16 && b_size <= 16 {
            if let (Some(a_val), Some(b_val)) = (a.as_u128(), b.as_u128()) {
                let result = a_val ^ b_val;
                
                if !a_signed && !b_signed {
                    if result <= u8::MAX as u128 { return NumericResult::U8(result as u8); }
                    if result <= u16::MAX as u128 { return NumericResult::U16(result as u16); }
                    if result <= u32::MAX as u128 { return NumericResult::U32(result as u32); }
                    if result <= u64::MAX as u128 { return NumericResult::U64(result as u64); }
                    return NumericResult::U128(result);
                }
                
                if a_signed || b_signed {
                    if let (Some(a_i), Some(b_i)) = (a.as_i128(), b.as_i128()) {
                        let result = a_i ^ b_i;
                        if result >= i8::MIN as i128 && result <= i8::MAX as i128 { 
                            return NumericResult::I8(result as i8); 
                        }
                        if result >= i16::MIN as i128 && result <= i16::MAX as i128 { 
                            return NumericResult::I16(result as i16); 
                        }
                        if result >= i32::MIN as i128 && result <= i32::MAX as i128 { 
                            return NumericResult::I32(result as i32); 
                        }
                        if result >= i64::MIN as i128 && result <= i64::MAX as i128 { 
                            return NumericResult::I64(result as i64); 
                        }
                        return NumericResult::I128(result);
                    }
                }
            }
        }
        
        let a_u256 = a.as_u256();
        let b_u256 = b.as_u256();
        let result = a_u256.0 ^ b_u256.0;
        NumericResult::U256(LuaU256(U256(result)))
    }

    fn smart_shl(a: &dyn NumericValue, shift: u32) -> Self {
        let a_size = a.size_bytes();
        let a_signed = a.is_signed();
        
        if a_size <= 16 {
            if let Some(a_val) = a.as_u128() {
                let result = a_val.checked_shl(shift).unwrap_or(0);
                
                if !a_signed {
                    if result <= u8::MAX as u128 { return NumericResult::U8(result as u8); }
                    if result <= u16::MAX as u128 { return NumericResult::U16(result as u16); }
                    if result <= u32::MAX as u128 { return NumericResult::U32(result as u32); }
                    if result <= u64::MAX as u128 { return NumericResult::U64(result as u64); }
                    return NumericResult::U128(result);
                }
                
                if let Some(a_i) = a.as_i128() {
                    let result = a_i.checked_shl(shift).unwrap_or(0);
                    if result >= i8::MIN as i128 && result <= i8::MAX as i128 { 
                        return NumericResult::I8(result as i8); 
                    }
                    if result >= i16::MIN as i128 && result <= i16::MAX as i128 { 
                        return NumericResult::I16(result as i16); 
                    }
                    if result >= i32::MIN as i128 && result <= i32::MAX as i128 { 
                        return NumericResult::I32(result as i32); 
                    }
                    if result >= i64::MIN as i128 && result <= i64::MAX as i128 { 
                        return NumericResult::I64(result as i64); 
                    }
                    return NumericResult::I128(result);
                }
            }
        }
        
        let a_u256 = a.as_u256();
        let result = a_u256.0 << shift;
        NumericResult::U256(LuaU256(U256(result)))
    }

    fn smart_shr(a: &dyn NumericValue, shift: u32) -> Self {
        let a_size = a.size_bytes();
        let a_signed = a.is_signed();
        
        if a_size <= 16 {
            if let Some(a_val) = a.as_u128() {
                let result = a_val >> shift;
                
                if !a_signed {
                    if result <= u8::MAX as u128 { return NumericResult::U8(result as u8); }
                    if result <= u16::MAX as u128 { return NumericResult::U16(result as u16); }
                    if result <= u32::MAX as u128 { return NumericResult::U32(result as u32); }
                    if result <= u64::MAX as u128 { return NumericResult::U64(result as u64); }
                    return NumericResult::U128(result);
                }
                
                if let Some(a_i) = a.as_i128() {
                    let result = a_i >> shift;
                    if result >= i8::MIN as i128 && result <= i8::MAX as i128 { 
                        return NumericResult::I8(result as i8); 
                    }
                    if result >= i16::MIN as i128 && result <= i16::MAX as i128 { 
                        return NumericResult::I16(result as i16); 
                    }
                    if result >= i32::MIN as i128 && result <= i32::MAX as i128 { 
                        return NumericResult::I32(result as i32); 
                    }
                    if result >= i64::MIN as i128 && result <= i64::MAX as i128 { 
                        return NumericResult::I64(result as i64); 
                    }
                    return NumericResult::I128(result);
                }
            }
        }
        
        let a_u256 = a.as_u256();
        let result = a_u256.0 >> shift;
        NumericResult::U256(LuaU256(U256(result)))
    }
}

impl IntoLua for NumericResult {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        match self {
            NumericResult::U8(v) => U8::new(v).into_lua(lua),
            NumericResult::U16(v) => U16::new(v).into_lua(lua),
            NumericResult::U32(v) => U32::new(v).into_lua(lua),
            NumericResult::U64(v) => U64::new(v).into_lua(lua),
            NumericResult::U128(v) => U128::new(v).into_lua(lua),
            NumericResult::U256(v) => v.into_lua(lua),
            NumericResult::I8(v) => I8::new(v).into_lua(lua),
            NumericResult::I16(v) => I16::new(v).into_lua(lua),
            NumericResult::I32(v) => I32::new(v).into_lua(lua),
            NumericResult::I64(v) => I64::new(v).into_lua(lua),
            NumericResult::I128(v) => I128::new(v).into_lua(lua),
        }
    }
}

impl<T> CrossCompare for T 
where 
    T: NumericValue 
{
    fn cross_cmp(&self, other: &dyn NumericValue) -> Option<Ordering> {
        let self_size = mem::size_of::<Self>();
        let other_size = other.size_bytes();
        let self_signed = self.is_signed();
        let other_signed = other.is_signed();

        match (self_signed, other_signed) {
            (false, false) => {
                if self_size <= 16 && other_size <= 16 {
                    match (self.as_u128(), other.as_u128()) {
                        (Some(a), Some(b)) => a.partial_cmp(&b),
                        _ => self.as_u256().0.partial_cmp(&other.as_u256().0),
                    }
                } else {
                    self.as_u256().0.partial_cmp(&other.as_u256().0)
                }
            },
            (true, true) => {
                match (self.as_i128(), other.as_i128()) {
                    (Some(a), Some(b)) => a.partial_cmp(&b),
                    _ => {
                        let self_u256 = self.as_u256();
                        let other_u256 = other.as_u256();
                        self_u256.0.partial_cmp(&other_u256.0)
                    }
                }
            },
            (false, true) => {
                if let Some(other_i128) = other.as_i128() {
                    if other_i128 < 0 {
                        return Some(Ordering::Greater);
                    }
                    if let Some(self_u128) = self.as_u128() {
                        return self_u128.partial_cmp(&(other_i128 as u128));
                    }
                }
                let self_u256 = self.as_u256();
                let other_u256 = other.as_u256();
                self_u256.0.partial_cmp(&other_u256.0)
            },
            (true, false) => {
                if let Some(self_i128) = self.as_i128() {
                    if self_i128 < 0 {
                        return Some(Ordering::Less);
                    }
                    if let Some(other_u128) = other.as_u128() {
                        return (self_i128 as u128).partial_cmp(&other_u128);
                    }
                }
                let self_u256 = self.as_u256();
                let other_u256 = other.as_u256();
                self_u256.0.partial_cmp(&other_u256.0)
            }
        }
    }

    fn cross_eq(&self, other: &dyn NumericValue) -> bool {
        matches!(self.cross_cmp(other), Some(Ordering::Equal))
    }
}

fn try_cross_eq<T: NumericValue + 'static>(this: &T, ud: &mlua::AnyUserData) -> Option<bool> {
    if let Ok(val) = ud.borrow::<U128>() { return Some(this.cross_eq(&*val)); }
    if let Ok(val) = ud.borrow::<U64>() { return Some(this.cross_eq(&*val)); }
    if let Ok(val) = ud.borrow::<U32>() { return Some(this.cross_eq(&*val)); }
    if let Ok(val) = ud.borrow::<U16>() { return Some(this.cross_eq(&*val)); }
    if let Ok(val) = ud.borrow::<U8>() { return Some(this.cross_eq(&*val)); }
    if let Ok(val) = ud.borrow::<I128>() { return Some(this.cross_eq(&*val)); }
    if let Ok(val) = ud.borrow::<I64>() { return Some(this.cross_eq(&*val)); }
    if let Ok(val) = ud.borrow::<I32>() { return Some(this.cross_eq(&*val)); }
    if let Ok(val) = ud.borrow::<I16>() { return Some(this.cross_eq(&*val)); }
    if let Ok(val) = ud.borrow::<I8>() { return Some(this.cross_eq(&*val)); }
    if let Ok(val) = ud.borrow::<LuaU256>() { return Some(this.cross_eq(&*val)); }
    None
}

fn try_cross_cmp<T: NumericValue + 'static>(this: &T, ud: &mlua::AnyUserData) -> Option<Option<Ordering>> {
    if let Ok(val) = ud.borrow::<U128>() { return Some(this.cross_cmp(&*val)); }
    if let Ok(val) = ud.borrow::<U64>() { return Some(this.cross_cmp(&*val)); }
    if let Ok(val) = ud.borrow::<U32>() { return Some(this.cross_cmp(&*val)); }
    if let Ok(val) = ud.borrow::<U16>() { return Some(this.cross_cmp(&*val)); }
    if let Ok(val) = ud.borrow::<U8>() { return Some(this.cross_cmp(&*val)); }
    if let Ok(val) = ud.borrow::<I128>() { return Some(this.cross_cmp(&*val)); }
    if let Ok(val) = ud.borrow::<I64>() { return Some(this.cross_cmp(&*val)); }
    if let Ok(val) = ud.borrow::<I32>() { return Some(this.cross_cmp(&*val)); }
    if let Ok(val) = ud.borrow::<I16>() { return Some(this.cross_cmp(&*val)); }
    if let Ok(val) = ud.borrow::<I8>() { return Some(this.cross_cmp(&*val)); }
    if let Ok(val) = ud.borrow::<LuaU256>() { return Some(this.cross_cmp(&*val)); }
    None
}

fn try_smart_add<T: NumericValue + 'static>(this: &T, ud: &mlua::AnyUserData) -> Option<NumericResult> {
    if let Ok(val) = ud.borrow::<U128>() { return Some(NumericResult::smart_add(this, &*val)); }
    if let Ok(val) = ud.borrow::<U64>() { return Some(NumericResult::smart_add(this, &*val)); }
    if let Ok(val) = ud.borrow::<U32>() { return Some(NumericResult::smart_add(this, &*val)); }
    if let Ok(val) = ud.borrow::<U16>() { return Some(NumericResult::smart_add(this, &*val)); }
    if let Ok(val) = ud.borrow::<U8>() { return Some(NumericResult::smart_add(this, &*val)); }
    if let Ok(val) = ud.borrow::<I128>() { return Some(NumericResult::smart_add(this, &*val)); }
    if let Ok(val) = ud.borrow::<I64>() { return Some(NumericResult::smart_add(this, &*val)); }
    if let Ok(val) = ud.borrow::<I32>() { return Some(NumericResult::smart_add(this, &*val)); }
    if let Ok(val) = ud.borrow::<I16>() { return Some(NumericResult::smart_add(this, &*val)); }
    if let Ok(val) = ud.borrow::<I8>() { return Some(NumericResult::smart_add(this, &*val)); }
    if let Ok(val) = ud.borrow::<LuaU256>() { return Some(NumericResult::smart_add(this, &*val)); }
    None
}

fn try_smart_sub<T: NumericValue + 'static>(this: &T, ud: &mlua::AnyUserData) -> Option<NumericResult> {
    if let Ok(val) = ud.borrow::<U128>() { return Some(NumericResult::smart_sub(this, &*val)); }
    if let Ok(val) = ud.borrow::<U64>() { return Some(NumericResult::smart_sub(this, &*val)); }
    if let Ok(val) = ud.borrow::<U32>() { return Some(NumericResult::smart_sub(this, &*val)); }
    if let Ok(val) = ud.borrow::<U16>() { return Some(NumericResult::smart_sub(this, &*val)); }
    if let Ok(val) = ud.borrow::<U8>() { return Some(NumericResult::smart_sub(this, &*val)); }
    if let Ok(val) = ud.borrow::<I128>() { return Some(NumericResult::smart_sub(this, &*val)); }
    if let Ok(val) = ud.borrow::<I64>() { return Some(NumericResult::smart_sub(this, &*val)); }
    if let Ok(val) = ud.borrow::<I32>() { return Some(NumericResult::smart_sub(this, &*val)); }
    if let Ok(val) = ud.borrow::<I16>() { return Some(NumericResult::smart_sub(this, &*val)); }
    if let Ok(val) = ud.borrow::<I8>() { return Some(NumericResult::smart_sub(this, &*val)); }
    if let Ok(val) = ud.borrow::<LuaU256>() { return Some(NumericResult::smart_sub(this, &*val)); }
    None
}

fn try_smart_mul<T: NumericValue + 'static>(this: &T, ud: &mlua::AnyUserData) -> Option<NumericResult> {
    if let Ok(val) = ud.borrow::<U128>() { return Some(NumericResult::smart_mul(this, &*val)); }
    if let Ok(val) = ud.borrow::<U64>() { return Some(NumericResult::smart_mul(this, &*val)); }
    if let Ok(val) = ud.borrow::<U32>() { return Some(NumericResult::smart_mul(this, &*val)); }
    if let Ok(val) = ud.borrow::<U16>() { return Some(NumericResult::smart_mul(this, &*val)); }
    if let Ok(val) = ud.borrow::<U8>() { return Some(NumericResult::smart_mul(this, &*val)); }
    if let Ok(val) = ud.borrow::<I128>() { return Some(NumericResult::smart_mul(this, &*val)); }
    if let Ok(val) = ud.borrow::<I64>() { return Some(NumericResult::smart_mul(this, &*val)); }
    if let Ok(val) = ud.borrow::<I32>() { return Some(NumericResult::smart_mul(this, &*val)); }
    if let Ok(val) = ud.borrow::<I16>() { return Some(NumericResult::smart_mul(this, &*val)); }
    if let Ok(val) = ud.borrow::<I8>() { return Some(NumericResult::smart_mul(this, &*val)); }
    if let Ok(val) = ud.borrow::<LuaU256>() { return Some(NumericResult::smart_mul(this, &*val)); }
    None
}

fn try_smart_div<T: NumericValue + 'static>(this: &T, ud: &mlua::AnyUserData) -> Option<NumericResult> {
    if let Ok(val) = ud.borrow::<U128>() { return Some(NumericResult::smart_div(this, &*val)); }
    if let Ok(val) = ud.borrow::<U64>() { return Some(NumericResult::smart_div(this, &*val)); }
    if let Ok(val) = ud.borrow::<U32>() { return Some(NumericResult::smart_div(this, &*val)); }
    if let Ok(val) = ud.borrow::<U16>() { return Some(NumericResult::smart_div(this, &*val)); }
    if let Ok(val) = ud.borrow::<U8>() { return Some(NumericResult::smart_div(this, &*val)); }
    if let Ok(val) = ud.borrow::<I128>() { return Some(NumericResult::smart_div(this, &*val)); }
    if let Ok(val) = ud.borrow::<I64>() { return Some(NumericResult::smart_div(this, &*val)); }
    if let Ok(val) = ud.borrow::<I32>() { return Some(NumericResult::smart_div(this, &*val)); }
    if let Ok(val) = ud.borrow::<I16>() { return Some(NumericResult::smart_div(this, &*val)); }
    if let Ok(val) = ud.borrow::<I8>() { return Some(NumericResult::smart_div(this, &*val)); }
    if let Ok(val) = ud.borrow::<LuaU256>() { return Some(NumericResult::smart_div(this, &*val)); }
    None
}

fn try_smart_bitand<T: NumericValue + 'static>(this: &T, ud: &mlua::AnyUserData) -> Option<NumericResult> {
    if let Ok(val) = ud.borrow::<U128>() { return Some(NumericResult::smart_bitand(this, &*val)); }
    if let Ok(val) = ud.borrow::<U64>() { return Some(NumericResult::smart_bitand(this, &*val)); }
    if let Ok(val) = ud.borrow::<U32>() { return Some(NumericResult::smart_bitand(this, &*val)); }
    if let Ok(val) = ud.borrow::<U16>() { return Some(NumericResult::smart_bitand(this, &*val)); }
    if let Ok(val) = ud.borrow::<U8>() { return Some(NumericResult::smart_bitand(this, &*val)); }
    if let Ok(val) = ud.borrow::<I128>() { return Some(NumericResult::smart_bitand(this, &*val)); }
    if let Ok(val) = ud.borrow::<I64>() { return Some(NumericResult::smart_bitand(this, &*val)); }
    if let Ok(val) = ud.borrow::<I32>() { return Some(NumericResult::smart_bitand(this, &*val)); }
    if let Ok(val) = ud.borrow::<I16>() { return Some(NumericResult::smart_bitand(this, &*val)); }
    if let Ok(val) = ud.borrow::<I8>() { return Some(NumericResult::smart_bitand(this, &*val)); }
    if let Ok(val) = ud.borrow::<LuaU256>() { return Some(NumericResult::smart_bitand(this, &*val)); }
    None
}

fn try_smart_bitor<T: NumericValue + 'static>(this: &T, ud: &mlua::AnyUserData) -> Option<NumericResult> {
    if let Ok(val) = ud.borrow::<U128>() { return Some(NumericResult::smart_bitor(this, &*val)); }
    if let Ok(val) = ud.borrow::<U64>() { return Some(NumericResult::smart_bitor(this, &*val)); }
    if let Ok(val) = ud.borrow::<U32>() { return Some(NumericResult::smart_bitor(this, &*val)); }
    if let Ok(val) = ud.borrow::<U16>() { return Some(NumericResult::smart_bitor(this, &*val)); }
    if let Ok(val) = ud.borrow::<U8>() { return Some(NumericResult::smart_bitor(this, &*val)); }
    if let Ok(val) = ud.borrow::<I128>() { return Some(NumericResult::smart_bitor(this, &*val)); }
    if let Ok(val) = ud.borrow::<I64>() { return Some(NumericResult::smart_bitor(this, &*val)); }
    if let Ok(val) = ud.borrow::<I32>() { return Some(NumericResult::smart_bitor(this, &*val)); }
    if let Ok(val) = ud.borrow::<I16>() { return Some(NumericResult::smart_bitor(this, &*val)); }
    if let Ok(val) = ud.borrow::<I8>() { return Some(NumericResult::smart_bitor(this, &*val)); }
    if let Ok(val) = ud.borrow::<LuaU256>() { return Some(NumericResult::smart_bitor(this, &*val)); }
    None
}

fn try_smart_bitxor<T: NumericValue + 'static>(this: &T, ud: &mlua::AnyUserData) -> Option<NumericResult> {
    if let Ok(val) = ud.borrow::<U128>() { return Some(NumericResult::smart_bitxor(this, &*val)); }
    if let Ok(val) = ud.borrow::<U64>() { return Some(NumericResult::smart_bitxor(this, &*val)); }
    if let Ok(val) = ud.borrow::<U32>() { return Some(NumericResult::smart_bitxor(this, &*val)); }
    if let Ok(val) = ud.borrow::<U16>() { return Some(NumericResult::smart_bitxor(this, &*val)); }
    if let Ok(val) = ud.borrow::<U8>() { return Some(NumericResult::smart_bitxor(this, &*val)); }
    if let Ok(val) = ud.borrow::<I128>() { return Some(NumericResult::smart_bitxor(this, &*val)); }
    if let Ok(val) = ud.borrow::<I64>() { return Some(NumericResult::smart_bitxor(this, &*val)); }
    if let Ok(val) = ud.borrow::<I32>() { return Some(NumericResult::smart_bitxor(this, &*val)); }
    if let Ok(val) = ud.borrow::<I16>() { return Some(NumericResult::smart_bitxor(this, &*val)); }
    if let Ok(val) = ud.borrow::<I8>() { return Some(NumericResult::smart_bitxor(this, &*val)); }
    if let Ok(val) = ud.borrow::<LuaU256>() { return Some(NumericResult::smart_bitxor(this, &*val)); }
    None
}

macro_rules! impl_numeric_value_unsigned {
    ($name:ident, $inner:ty) => {
        impl NumericValue for $name {
            fn as_u128(&self) -> Option<u128> {
                if mem::size_of::<$inner>() <= 16 {
                    Some(self.0 as u128)
                } else {
                    None
                }
            }

            fn as_i128(&self) -> Option<i128> {
                None
            }

            fn as_u256(&self) -> U256 {
                U256(PrimitiveU256::from(self.0 as u128))
            }

            fn is_signed(&self) -> bool {
                false
            }

            fn size_bytes(&self) -> usize {
                mem::size_of::<$inner>()
            }

            fn type_name(&self) -> &'static str {
                stringify!($name)
            }
        }
    };
}

macro_rules! impl_numeric_value_signed {
    ($name:ident, $inner:ty) => {
        impl NumericValue for $name {
            fn as_u128(&self) -> Option<u128> {
                if self.0 >= 0 && mem::size_of::<$inner>() <= 16 {
                    Some(self.0 as u128)
                } else {
                    None
                }
            }

            fn as_i128(&self) -> Option<i128> {
                if mem::size_of::<$inner>() <= 16 {
                    Some(self.0 as i128)
                } else {
                    None
                }
            }

            fn as_u256(&self) -> U256 {
                if self.0 >= 0 {
                    U256(PrimitiveU256::from(self.0 as u128))
                } else {
                    U256(PrimitiveU256::zero())
                }
            }

            fn is_signed(&self) -> bool {
                true
            }

            fn size_bytes(&self) -> usize {
                mem::size_of::<$inner>()
            }

            fn type_name(&self) -> &'static str {
                stringify!($name)
            }
        }
    };
}

impl NumericValue for LuaU256 {
    fn as_u128(&self) -> Option<u128> {
        if self.0.0 <= PrimitiveU256::from(u128::MAX) {
            Some(self.0.0.as_u128())
        } else {
            None
        }
    }

    fn as_i128(&self) -> Option<i128> {
        None
    }

    fn as_u256(&self) -> U256 {
        self.0
    }

    fn is_signed(&self) -> bool {
        false
    }

    fn size_bytes(&self) -> usize {
        32
    }

    fn type_name(&self) -> &'static str {
        "LuaU256"
    }
}

macro_rules! add_cross_comparison_methods {
    ($methods:ident, $name:ident, [$($type:ident),*]) => {
        $methods.add_meta_method(MetaMethod::Eq, |_, this, other: Value| {
            match other {
                Value::UserData(ud) => Ok(try_cross_eq(this, &ud).unwrap_or(false)),
                _ => Ok(false),
            }
        });

        $methods.add_meta_method(MetaMethod::Lt, |_, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(cmp_result) = try_cross_cmp(this, &ud) {
                        Ok(matches!(cmp_result, Some(Ordering::Less)))
                    } else {
                        Err(mlua::Error::RuntimeError(format!("Cannot compare {} with unknown type", this.type_name())))
                    }
                },
                _ => Err(mlua::Error::RuntimeError(format!("Cannot compare {} with {}", this.type_name(), other.type_name()))),
            }
        });

        $methods.add_meta_method(MetaMethod::Le, |_, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(cmp_result) = try_cross_cmp(this, &ud) {
                        Ok(matches!(cmp_result, Some(Ordering::Less | Ordering::Equal)))
                    } else {
                        Err(mlua::Error::RuntimeError(format!("Cannot compare {} with unknown type", this.type_name())))
                    }
                },
                _ => Err(mlua::Error::RuntimeError(format!("Cannot compare {} with {}", this.type_name(), other.type_name()))),
            }
        });
    };
}

macro_rules! add_smart_arithmetic_methods {
    ($methods:ident, $name:ident, [$($type:ident),*]) => {
        $methods.add_meta_method(MetaMethod::Add, |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_add(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot add with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot add with non-numeric type".to_string())),
            }
        });

        $methods.add_meta_method(MetaMethod::Sub, |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_sub(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot subtract with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot subtract with non-numeric type".to_string())),
            }
        });

        $methods.add_meta_method(MetaMethod::Mul, |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_mul(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot multiply with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot multiply with non-numeric type".to_string())),
            }
        });

        $methods.add_meta_method(MetaMethod::Div, |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_div(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot divide with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot divide with non-numeric type".to_string())),
            }
        });
    };
}

/*macro_rules! add_smart_bitwise_methods {
    ($methods:ident, $name:ident, [$($type:ident),*]) => {
        $methods.add_meta_method(MetaMethod::BAnd, |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_bitand(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot perform bitwise AND with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot perform bitwise AND with non-numeric type".to_string())),
            }
        });

        $methods.add_meta_method(MetaMethod::BOr, |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_bitor(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot perform bitwise OR with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot perform bitwise OR with non-numeric type".to_string())),
            }
        });

        $methods.add_meta_method(MetaMethod::BXor, |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_bitxor(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot perform bitwise XOR with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot perform bitwise XOR with non-numeric type".to_string())),
            }
        });

        $methods.add_meta_method(MetaMethod::Shl, |lua, this, shift: u32| {
            NumericResult::smart_shl(this, shift).into_lua(lua)
        });

        $methods.add_meta_method(MetaMethod::Shr, |lua, this, shift: u32| {
            NumericResult::smart_shr(this, shift).into_lua(lua)
        });

        $methods.add_meta_method(MetaMethod::BNot, |lua, this, _: ()| {
            let a_size = this.size_bytes();
            let a_signed = this.is_signed();
            
            if a_size <= 16 {
                if let Some(a_val) = this.as_u128() {
                    let result = !a_val;
                    
                    if !a_signed {
                        if a_size == 1 { return U8::new(result as u8).into_lua(lua); }
                        if a_size == 2 { return U16::new(result as u16).into_lua(lua); }
                        if a_size == 4 { return U32::new(result as u32).into_lua(lua); }
                        if a_size == 8 { return U64::new(result as u64).into_lua(lua); }
                        return U128::new(result).into_lua(lua);
                    }
                    
                    if let Some(a_i) = this.as_i128() {
                        let result = !a_i;
                        if a_size == 1 { return I8::new(result as i8).into_lua(lua); }
                        if a_size == 2 { return I16::new(result as i16).into_lua(lua); }
                        if a_size == 4 { return I32::new(result as i32).into_lua(lua); }
                        if a_size == 8 { return I64::new(result as i64).into_lua(lua); }
                        return I128::new(result).into_lua(lua);
                    }
                }
            }
            
            let a_u256 = this.as_u256();
            let result = !a_u256.0;
            LuaU256(U256(result)).into_lua(lua)
        });
    };
}*/

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Decode, Encode)]
pub struct LuaU256(pub U256);

impl LuaU256 {
    pub fn new(value: U256) -> Self {
        Self(value)
    }

    pub fn from_u128(value: u128) -> Self {
        Self(U256(PrimitiveU256::from(value)))
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match PrimitiveU256::from_dec_str(s) {
            Ok(val) => Ok(Self(U256(val))),
            Err(_) => Err(format!("Invalid U256 string: {}", s))
        }
    }

    pub fn saturating_add(self, other: Self) -> Self {
        Self(U256(self.0.0.saturating_add(other.0.0)))
    }

    pub fn saturating_sub(self, other: Self) -> Self {
        Self(U256(self.0.0.saturating_sub(other.0.0)))
    }

    pub fn saturating_mul(self, other: Self) -> Self {
        Self(U256(self.0.0.saturating_mul(other.0.0)))
    }

    pub fn saturating_div(self, other: Self) -> Self {
        if other.0.0.is_zero() {
            Self(U256(PrimitiveU256::MAX))
        } else {
            Self(U256(self.0.0 / other.0.0))
        }
    }

    pub fn saturating_pow(self, exp: u32) -> Self {
        Self(U256(self.0.0.checked_pow(exp.into()).unwrap_or(PrimitiveU256::MAX)))
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.0.checked_add(other.0.0).map(|val| Self(U256(val)))
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.0.checked_sub(other.0.0).map(|val| Self(U256(val)))
    }

    pub fn checked_mul(self, other: Self) -> Option<Self> {
        self.0.0.checked_mul(other.0.0).map(|val| Self(U256(val)))
    }

    pub fn checked_div(self, other: Self) -> Option<Self> {
        if other.0.0.is_zero() {
            None
        } else {
            Some(Self(U256(self.0.0 / other.0.0)))
        }
    }

    pub fn wrapping_add(self, other: Self) -> Self {
        Self(U256(self.0.0.overflowing_add(other.0.0).0))
    }

    pub fn wrapping_sub(self, other: Self) -> Self {
        Self(U256(self.0.0.overflowing_sub(other.0.0).0))
    }

    pub fn wrapping_mul(self, other: Self) -> Self {
        Self(U256(self.0.0.overflowing_mul(other.0.0).0))
    }

    pub fn bitand(self, other: Self) -> Self {
        Self(U256(self.0.0 & other.0.0))
    }

    pub fn bitor(self, other: Self) -> Self {
        Self(U256(self.0.0 | other.0.0))
    }

    pub fn bitxor(self, other: Self) -> Self {
        Self(U256(self.0.0 ^ other.0.0))
    }

    pub fn bitnot(self) -> Self {
        Self(U256(!self.0.0))
    }

    pub fn shl(self, shift: u32) -> Self {
        Self(U256(self.0.0 << shift))
    }

    pub fn shr(self, shift: u32) -> Self {
        Self(U256(self.0.0 >> shift))
    }

    pub fn value(self) -> U256 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0.0.is_zero()
    }

    pub fn is_max(self) -> bool {
        self.0.0 == PrimitiveU256::MAX
    }

    pub fn min_value() -> Self {
        Self(U256(PrimitiveU256::zero()))
    }

    pub fn max_value() -> Self {
        Self(U256(PrimitiveU256::MAX))
    }

    pub fn leading_zeros(self) -> u32 {
        self.0.0.leading_zeros()
    }

    pub fn trailing_zeros(self) -> u32 {
        self.0.0.trailing_zeros()
    }

    pub fn count_ones(self) -> u32 {
        0
    }
}

impl fmt::Display for LuaU256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.0)
    }
}

impl FromLua for LuaU256 {
    fn from_lua(value: Value, _lua: &Lua) -> LuaResult<Self> {
        match value {
            Value::UserData(ud) => {
                Ok(*ud.borrow::<LuaU256>()?)
            }
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "U256".to_string(),
                message: Some("Expected U256 userdata".to_string()),
            }),
        }
    }
}

impl UserData for LuaU256 {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |_, value: u128| Ok(LuaU256::from_u128(value)));
        methods.add_function("from_string", |_, s: String| LuaU256::from_str(&s).map_err(|e| mlua::Error::RuntimeError(e)));
        methods.add_function("min", |_, _: ()| Ok(LuaU256::min_value()));
        methods.add_function("max", |_, _: ()| Ok(LuaU256::max_value()));

        methods.add_method("add", |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_add(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot add with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot add with non-numeric type".to_string())),
            }
        });

        methods.add_method("sub", |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_sub(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot subtract with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot subtract with non-numeric type".to_string())),
            }
        });

        methods.add_method("mul", |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_mul(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot multiply with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot multiply with non-numeric type".to_string())),
            }
        });

        methods.add_method("div", |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_div(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot divide with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot divide with non-numeric type".to_string())),
            }
        });

        methods.add_method("bitand", |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_bitand(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot perform bitwise AND with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot perform bitwise AND with non-numeric type".to_string())),
            }
        });

        methods.add_method("bitor", |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_bitor(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot perform bitwise OR with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot perform bitwise OR with non-numeric type".to_string())),
            }
        });

        methods.add_method("bitxor", |lua, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    if let Some(result) = try_smart_bitxor(this, &ud) {
                        result.into_lua(lua)
                    } else {
                        Err(mlua::Error::RuntimeError("Cannot perform bitwise XOR with unknown type".to_string()))
                    }
                },
                _ => Err(mlua::Error::RuntimeError("Cannot perform bitwise XOR with non-numeric type".to_string())),
            }
        });

        methods.add_method("bitnot", |_, this, _: ()| Ok(this.bitnot()));

        methods.add_method("shl", |lua, this, shift: u32| {
            NumericResult::smart_shl(this, shift).into_lua(lua)
        });

        methods.add_method("shr", |lua, this, shift: u32| {
            NumericResult::smart_shr(this, shift).into_lua(lua)
        });

        methods.add_method("is_zero", |_, this, _: ()| Ok(this.is_zero()));
        methods.add_method("is_max", |_, this, _: ()| Ok(this.is_max()));
        methods.add_method("leading_zeros", |_, this, _: ()| Ok(this.leading_zeros()));
        methods.add_method("trailing_zeros", |_, this, _: ()| Ok(this.trailing_zeros()));
        methods.add_method("count_ones", |_, this, _: ()| Ok(this.count_ones()));
        methods.add_method("to_string", |_, this, _: ()| Ok(this.to_string()));

        methods.add_meta_method(MetaMethod::ToString, |_, this, _: ()| Ok(this.to_string()));

        add_cross_comparison_methods!(methods, LuaU256, [U128, U64, U32, U16, U8, I128, I64, I32, I16, I8]);
        add_smart_arithmetic_methods!(methods, LuaU256, [U128, U64, U32, U16, U8, I128, I64, I32, I16, I8, LuaU256]);
    }
}

pub fn register_luau256_library(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();
    let table = lua.create_table()?;
    
    table.set("new", lua.create_function(|_, value: u128| Ok(LuaU256::from_u128(value)))?)?;
    table.set("from_string", lua.create_function(|_, s: String| LuaU256::from_str(&s).map_err(|e| mlua::Error::RuntimeError(e)))?)?;
    table.set("min", lua.create_function(|_, _: ()| Ok(LuaU256::min_value()))?)?;
    table.set("max", lua.create_function(|_, _: ()| Ok(LuaU256::max_value()))?)?;
    
    globals.set("U256", table)?;
    Ok(())
}

macro_rules! define_number_type {
    ($name:ident, $inner:ty, $lua_name:expr) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Decode, Encode, Serialize, Deserialize)]
        pub struct $name(pub $inner);

        impl $name {
            pub fn new(value: $inner) -> Self {
                Self(value)
            }

            pub fn from_str(s: &str) -> Result<Self, std::num::ParseIntError> {
                Ok(Self(s.parse::<$inner>()?))
            }

            pub fn value(self) -> $inner {
                self.0
            }

            pub fn is_zero(self) -> bool {
                self.0 == 0
            }

            pub fn is_max(self) -> bool {
                self.0 == <$inner>::MAX
            }

            pub fn min_value() -> Self {
                Self(<$inner>::MIN)
            }

            pub fn max_value() -> Self {
                Self(<$inner>::MAX)
            }

            pub fn leading_zeros(self) -> u32 {
                self.0.leading_zeros()
            }

            pub fn trailing_zeros(self) -> u32 {
                self.0.trailing_zeros()
            }

            pub fn count_ones(self) -> u32 {
                self.0.count_ones()
            }

            pub fn reverse_bits(self) -> Self {
                Self(self.0.reverse_bits())
            }

            pub fn swap_bytes(self) -> Self {
                Self(self.0.swap_bytes())
            }

            pub fn bitnot(self) -> Self {
                Self(!self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromLua for $name {
            fn from_lua(value: Value, _lua: &Lua) -> LuaResult<Self> {
                match value {
                    Value::UserData(ud) => {
                        Ok(*ud.borrow::<$name>()?)
                    }
                    _ => Err(mlua::Error::FromLuaConversionError {
                        from: value.type_name(),
                        to: $lua_name.to_string(),
                        message: Some(format!("Expected {} userdata", $lua_name)),
                    }),
                }
            }
        }

        impl UserData for $name {
            fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
                methods.add_function("new", |_, value: $inner| Ok($name::new(value)));
                methods.add_function("from_string", |_, s: String| $name::from_str(&s).map_err(|e| mlua::Error::RuntimeError(format!("Invalid {} string: {}", $lua_name, e))));
                methods.add_function("min", |_, _: ()| Ok($name::min_value()));
                methods.add_function("max", |_, _: ()| Ok($name::max_value()));

                methods.add_method("add", |lua, this, other: Value| {
                    match other {
                        Value::UserData(ud) => {
                            if let Some(result) = try_smart_add(this, &ud) {
                                result.into_lua(lua)
                            } else {
                                Err(mlua::Error::RuntimeError("Cannot add with unknown type".to_string()))
                            }
                        },
                        _ => Err(mlua::Error::RuntimeError("Cannot add with non-numeric type".to_string())),
                    }
                });

                methods.add_method("sub", |lua, this, other: Value| {
                    match other {
                        Value::UserData(ud) => {
                            if let Some(result) = try_smart_sub(this, &ud) {
                                result.into_lua(lua)
                            } else {
                                Err(mlua::Error::RuntimeError("Cannot subtract with unknown type".to_string()))
                            }
                        },
                        _ => Err(mlua::Error::RuntimeError("Cannot subtract with non-numeric type".to_string())),
                    }
                });

                methods.add_method("mul", |lua, this, other: Value| {
                    match other {
                        Value::UserData(ud) => {
                            if let Some(result) = try_smart_mul(this, &ud) {
                                result.into_lua(lua)
                            } else {
                                Err(mlua::Error::RuntimeError("Cannot multiply with unknown type".to_string()))
                            }
                        },
                        _ => Err(mlua::Error::RuntimeError("Cannot multiply with non-numeric type".to_string())),
                    }
                });

                methods.add_method("div", |lua, this, other: Value| {
                    match other {
                        Value::UserData(ud) => {
                            if let Some(result) = try_smart_div(this, &ud) {
                                result.into_lua(lua)
                            } else {
                                Err(mlua::Error::RuntimeError("Cannot divide with unknown type".to_string()))
                            }
                        },
                        _ => Err(mlua::Error::RuntimeError("Cannot divide with non-numeric type".to_string())),
                    }
                });

                methods.add_method("bitand", |lua, this, other: Value| {
                    match other {
                        Value::UserData(ud) => {
                            if let Some(result) = try_smart_bitand(this, &ud) {
                                result.into_lua(lua)
                            } else {
                                Err(mlua::Error::RuntimeError("Cannot perform bitwise AND with unknown type".to_string()))
                            }
                        },
                        _ => Err(mlua::Error::RuntimeError("Cannot perform bitwise AND with non-numeric type".to_string())),
                    }
                });

                methods.add_method("bitor", |lua, this, other: Value| {
                    match other {
                        Value::UserData(ud) => {
                            if let Some(result) = try_smart_bitor(this, &ud) {
                                result.into_lua(lua)
                            } else {
                                Err(mlua::Error::RuntimeError("Cannot perform bitwise OR with unknown type".to_string()))
                            }
                        },
                        _ => Err(mlua::Error::RuntimeError("Cannot perform bitwise OR with non-numeric type".to_string())),
                    }
                });

                methods.add_method("bitxor", |lua, this, other: Value| {
                    match other {
                        Value::UserData(ud) => {
                            if let Some(result) = try_smart_bitxor(this, &ud) {
                                result.into_lua(lua)
                            } else {
                                Err(mlua::Error::RuntimeError("Cannot perform bitwise XOR with unknown type".to_string()))
                            }
                        },
                        _ => Err(mlua::Error::RuntimeError("Cannot perform bitwise XOR with non-numeric type".to_string())),
                    }
                });

                methods.add_method("bitnot", |_, this, _: ()| Ok(this.bitnot()));

                methods.add_method("shl", |lua, this, shift: u32| {
                    NumericResult::smart_shl(this, shift).into_lua(lua)
                });

                methods.add_method("shr", |lua, this, shift: u32| {
                    NumericResult::smart_shr(this, shift).into_lua(lua)
                });

                methods.add_method("value", |_, this, _: ()| Ok(this.value()));
                methods.add_method("is_zero", |_, this, _: ()| Ok(this.is_zero()));
                methods.add_method("is_max", |_, this, _: ()| Ok(this.is_max()));
                methods.add_method("leading_zeros", |_, this, _: ()| Ok(this.leading_zeros()));
                methods.add_method("trailing_zeros", |_, this, _: ()| Ok(this.trailing_zeros()));
                methods.add_method("count_ones", |_, this, _: ()| Ok(this.count_ones()));
                methods.add_method("reverse_bits", |_, this, _: ()| Ok(this.reverse_bits()));
                methods.add_method("swap_bytes", |_, this, _: ()| Ok(this.swap_bytes()));
                methods.add_method("to_string", |_, this, _: ()| Ok(this.to_string()));

                methods.add_meta_method(MetaMethod::ToString, |_, this, _: ()| Ok(this.to_string()));

                add_cross_comparison_methods!(methods, $name, [U128, U64, U32, U16, U8, I128, I64, I32, I16, I8, LuaU256]);
                add_smart_arithmetic_methods!(methods, $name, [U128, U64, U32, U16, U8, I128, I64, I32, I16, I8, LuaU256]);
            }
        }

        paste::paste! {
            pub fn [<register_ $name:lower _library>](lua: &Lua) -> LuaResult<()> {
                let globals = lua.globals();
                let table = lua.create_table()?;
                
                table.set("new", lua.create_function(|_, value: $inner| Ok($name::new(value)))?)?;
                table.set("from_string", lua.create_function(|_, s: String| $name::from_str(&s).map_err(|e| mlua::Error::RuntimeError(format!("Invalid {} string: {}", $lua_name, e))))?)?;
                table.set("min", lua.create_function(|_, _: ()| Ok($name::min_value()))?)?;
                table.set("max", lua.create_function(|_, _: ()| Ok($name::max_value()))?)?;
                
                globals.set($lua_name, table)?;
                Ok(())
            }

            pub fn [<register_ $name:lower _shorthand>](lua: &Lua) -> LuaResult<()> {
                let globals = lua.globals();
                let shorthand = $lua_name.to_lowercase();
                globals.set(shorthand.as_str(), lua.create_function(|_, value: mlua::Value| {
                    match value {
                        mlua::Value::Integer(n) => {
                            if n >= <$inner>::MIN as mlua::Integer && n <= <$inner>::MAX as mlua::Integer {
                                Ok($name::new(n as $inner))
                            } else {
                                Err(mlua::Error::RuntimeError(format!("{} value {} out of range", stringify!($name).to_lowercase(), n)))
                            }
                        },
                        mlua::Value::Number(n) => {
                            if n >= <$inner>::MIN as f64 && n <= <$inner>::MAX as f64 {
                                Ok($name::new(n as $inner))
                            } else {
                                Err(mlua::Error::RuntimeError(format!("{} value {} out of range", stringify!($name).to_lowercase(), n)))
                            }
                        },
                        mlua::Value::String(s) => {
                            let s = s.to_str()?;
                            $name::from_str(&s).map_err(|e| mlua::Error::RuntimeError(format!("Invalid {}: {}", stringify!($name).to_lowercase(), e)))
                        },
                        mlua::Value::Nil => Ok($name::new(0)),
                        _ => Err(mlua::Error::RuntimeError(format!("{}() expects number or string", stringify!($name).to_lowercase())))
                    }
                })?)?;
                Ok(())
            }
        }
    };
}

define_number_type!(U128, u128, "U128");
define_number_type!(U64, u64, "U64");
define_number_type!(U32, u32, "U32");
define_number_type!(U16, u16, "U16");
define_number_type!(U8, u8, "U8");

define_number_type!(I128, i128, "I128");
define_number_type!(I64, i64, "I64");
define_number_type!(I32, i32, "I32");
define_number_type!(I16, i16, "I16");
define_number_type!(I8, i8, "I8");

impl_numeric_value_unsigned!(U128, u128);
impl_numeric_value_unsigned!(U64, u64);
impl_numeric_value_unsigned!(U32, u32);
impl_numeric_value_unsigned!(U16, u16);
impl_numeric_value_unsigned!(U8, u8);

impl_numeric_value_signed!(I128, i128);
impl_numeric_value_signed!(I64, i64);
impl_numeric_value_signed!(I32, i32);
impl_numeric_value_signed!(I16, i16);
impl_numeric_value_signed!(I8, i8);

pub fn register_luau256_shorthand(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();
    globals.set("u256", lua.create_function(|_, value: mlua::Value| {
        match value {
            mlua::Value::Integer(n) => {
                if n >= 0 {
                    Ok(LuaU256::from_u128(n as u128))
                } else {
                    Err(mlua::Error::RuntimeError(format!("u256 value {} cannot be negative", n)))
                }
            },
            mlua::Value::Number(n) => {
                if n >= 0.0 && n <= u128::MAX as f64 {
                    Ok(LuaU256::from_u128(n as u128))
                } else {
                    Err(mlua::Error::RuntimeError(format!("u256 value {} out of range", n)))
                }
            },
            mlua::Value::String(s) => {
                let s = s.to_str()?;
                LuaU256::from_str(&s).map_err(|e| mlua::Error::RuntimeError(format!("Invalid u256: {}", e)))
            },
            mlua::Value::Nil => Ok(LuaU256::from_u128(0)),
            _ => Err(mlua::Error::RuntimeError("u256() expects number or string".to_string()))
        }
    })?)?;
    Ok(())
}

macro_rules! register_all_numeric_types {
    ($($name:ident),*) => {
        pub fn register_all_numeric_libraries(lua: &Lua) -> LuaResult<()> {
            info!("Registering numeric libraries");
            $(
                paste::paste! {
                    [<register_ $name:lower _library>](lua)?;
                }
            )*
            
            info!("Registering shorthand functions");
            $(
                paste::paste! {
                    [<register_ $name:lower _shorthand>](lua)?;
                }
            )*
            
            Ok(())
        }
    };
}

register_all_numeric_types![
    LuaU256, U128, U64, U32, U16, U8, I128, I64, I32, I16, I8
];