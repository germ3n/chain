use {
    mlua::{Lua, UserData, UserDataMethods, Result as LuaResult, MetaMethod, Value, FromLua},
    std::{fmt, mem, cmp::Ordering},
    bincode::{Decode, Encode},
    serde::{Serialize, Deserialize},
    crate::structs::num::U256,
    primitive_types::U256 as PrimitiveU256,
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
                Value::UserData(ud) => {
                    $(
                        if let Ok(other_val) = ud.borrow::<$type>() {
                            return Ok(this.cross_eq(&*other_val));
                        }
                    )*
                    Ok(false)
                },
                _ => Ok(false),
            }
        });

        $methods.add_meta_method(MetaMethod::Lt, |_, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    $(
                        if let Ok(other_val) = ud.borrow::<$type>() {
                            return Ok(matches!(this.cross_cmp(&*other_val), Some(Ordering::Less)));
                        }
                    )*
                    Err(mlua::Error::RuntimeError(format!("Cannot compare {} with unknown type", this.type_name())))
                },
                _ => Err(mlua::Error::RuntimeError(format!("Cannot compare {} with {}", this.type_name(), other.type_name()))),
            }
        });

        $methods.add_meta_method(MetaMethod::Le, |_, this, other: Value| {
            match other {
                Value::UserData(ud) => {
                    $(
                        if let Ok(other_val) = ud.borrow::<$type>() {
                            return Ok(matches!(this.cross_cmp(&*other_val), Some(Ordering::Less | Ordering::Equal)));
                        }
                    )*
                    Err(mlua::Error::RuntimeError(format!("Cannot compare {} with unknown type", this.type_name())))
                },
                _ => Err(mlua::Error::RuntimeError(format!("Cannot compare {} with {}", this.type_name(), other.type_name()))),
            }
        });
    };
}

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
        //self.0.0.count_ones()
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

        methods.add_method("saturating_add", |_, this, other: LuaU256| Ok(this.saturating_add(other)));
        methods.add_method("saturating_sub", |_, this, other: LuaU256| Ok(this.saturating_sub(other)));
        methods.add_method("saturating_mul", |_, this, other: LuaU256| Ok(this.saturating_mul(other)));
        methods.add_method("saturating_div", |_, this, other: LuaU256| Ok(this.saturating_div(other)));
        methods.add_method("saturating_pow", |_, this, exp: u32| Ok(this.saturating_pow(exp)));
        methods.add_method("checked_add", |_, this, other: LuaU256| Ok(this.checked_add(other)));
        methods.add_method("checked_sub", |_, this, other: LuaU256| Ok(this.checked_sub(other)));
        methods.add_method("checked_mul", |_, this, other: LuaU256| Ok(this.checked_mul(other)));
        methods.add_method("checked_div", |_, this, other: LuaU256| Ok(this.checked_div(other)));
        methods.add_method("wrapping_add", |_, this, other: LuaU256| Ok(this.wrapping_add(other)));
        methods.add_method("wrapping_sub", |_, this, other: LuaU256| Ok(this.wrapping_sub(other)));
        methods.add_method("wrapping_mul", |_, this, other: LuaU256| Ok(this.wrapping_mul(other)));

        //methods.add_method("value", |_, this, _: ()| Ok(this.value()));
        methods.add_method("is_zero", |_, this, _: ()| Ok(this.is_zero()));
        methods.add_method("is_max", |_, this, _: ()| Ok(this.is_max()));
        methods.add_method("leading_zeros", |_, this, _: ()| Ok(this.leading_zeros()));
        methods.add_method("trailing_zeros", |_, this, _: ()| Ok(this.trailing_zeros()));
        methods.add_method("count_ones", |_, this, _: ()| Ok(this.count_ones()));
        methods.add_method("to_string", |_, this, _: ()| Ok(this.to_string()));

        methods.add_meta_method(MetaMethod::Add, |_, this, other: LuaU256| Ok(this.saturating_add(other)));
        methods.add_meta_method(MetaMethod::Sub, |_, this, other: LuaU256| Ok(this.saturating_sub(other)));
        methods.add_meta_method(MetaMethod::Mul, |_, this, other: LuaU256| Ok(this.saturating_mul(other)));
        methods.add_meta_method(MetaMethod::Div, |_, this, other: LuaU256| Ok(this.saturating_div(other)));
        methods.add_meta_method(MetaMethod::ToString, |_, this, _: ()| Ok(this.to_string()));

        add_cross_comparison_methods!(methods, LuaU256, [U128, U64, U32, U16, U8, I128, I64, I32, I16, I8]);
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

            pub fn saturating_add(self, other: Self) -> Self {
                Self(self.0.saturating_add(other.0))
            }

            pub fn saturating_sub(self, other: Self) -> Self {
                Self(self.0.saturating_sub(other.0))
            }

            pub fn saturating_mul(self, other: Self) -> Self {
                Self(self.0.saturating_mul(other.0))
            }

            pub fn saturating_div(self, other: Self) -> Self {
                if other.0 == 0 {
                    Self(<$inner>::MAX)
                } else {
                    Self(self.0 / other.0)
                }
            }

            pub fn saturating_pow(self, exp: u32) -> Self {
                Self(self.0.saturating_pow(exp))
            }

            pub fn checked_add(self, other: Self) -> Option<Self> {
                self.0.checked_add(other.0).map(Self)
            }

            pub fn checked_sub(self, other: Self) -> Option<Self> {
                self.0.checked_sub(other.0).map(Self)
            }

            pub fn checked_mul(self, other: Self) -> Option<Self> {
                self.0.checked_mul(other.0).map(Self)
            }

            pub fn checked_div(self, other: Self) -> Option<Self> {
                if other.0 == 0 {
                    None
                } else {
                    Some(Self(self.0 / other.0))
                }
            }

            pub fn wrapping_add(self, other: Self) -> Self {
                Self(self.0.wrapping_add(other.0))
            }

            pub fn wrapping_sub(self, other: Self) -> Self {
                Self(self.0.wrapping_sub(other.0))
            }

            pub fn wrapping_mul(self, other: Self) -> Self {
                Self(self.0.wrapping_mul(other.0))
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

                methods.add_method("saturating_add", |_, this, other: $name| Ok(this.saturating_add(other)));
                methods.add_method("saturating_sub", |_, this, other: $name| Ok(this.saturating_sub(other)));
                methods.add_method("saturating_mul", |_, this, other: $name| Ok(this.saturating_mul(other)));
                methods.add_method("saturating_div", |_, this, other: $name| Ok(this.saturating_div(other)));
                methods.add_method("saturating_pow", |_, this, exp: u32| Ok(this.saturating_pow(exp)));
                methods.add_method("checked_add", |_, this, other: $name| Ok(this.checked_add(other)));
                methods.add_method("checked_sub", |_, this, other: $name| Ok(this.checked_sub(other)));
                methods.add_method("checked_mul", |_, this, other: $name| Ok(this.checked_mul(other)));
                methods.add_method("checked_div", |_, this, other: $name| Ok(this.checked_div(other)));
                methods.add_method("wrapping_add", |_, this, other: $name| Ok(this.wrapping_add(other)));
                methods.add_method("wrapping_sub", |_, this, other: $name| Ok(this.wrapping_sub(other)));
                methods.add_method("wrapping_mul", |_, this, other: $name| Ok(this.wrapping_mul(other)));

                methods.add_method("value", |_, this, _: ()| Ok(this.value()));
                methods.add_method("is_zero", |_, this, _: ()| Ok(this.is_zero()));
                methods.add_method("is_max", |_, this, _: ()| Ok(this.is_max()));
                methods.add_method("leading_zeros", |_, this, _: ()| Ok(this.leading_zeros()));
                methods.add_method("trailing_zeros", |_, this, _: ()| Ok(this.trailing_zeros()));
                methods.add_method("count_ones", |_, this, _: ()| Ok(this.count_ones()));
                methods.add_method("reverse_bits", |_, this, _: ()| Ok(this.reverse_bits()));
                methods.add_method("swap_bytes", |_, this, _: ()| Ok(this.swap_bytes()));
                methods.add_method("to_string", |_, this, _: ()| Ok(this.to_string()));

                methods.add_meta_method(MetaMethod::Add, |_, this, other: $name| Ok(this.saturating_add(other)));
                methods.add_meta_method(MetaMethod::Sub, |_, this, other: $name| Ok(this.saturating_sub(other)));
                methods.add_meta_method(MetaMethod::Mul, |_, this, other: $name| Ok(this.saturating_mul(other)));
                methods.add_meta_method(MetaMethod::Div, |_, this, other: $name| Ok(this.saturating_div(other)));
                methods.add_meta_method(MetaMethod::ToString, |_, this, _: ()| Ok(this.to_string()));

                add_cross_comparison_methods!(methods, $name, [U128, U64, U32, U16, U8, I128, I64, I32, I16, I8, LuaU256]);
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