use crate::RawLuaString;

#[cfg(feature = "lua51")]
pub const LUA_CONSTANT_NIL: u8 = 0;
#[cfg(feature = "lua51")]
pub const LUA_CONSTANT_BOOLEAN: u8 = 1;
#[cfg(feature = "lua51")]
pub const LUA_CONSTANT_NUMBER: u8 = 3;
#[cfg(feature = "lua51")]
pub const LUA_CONSTANT_STRING: u8 = 4;

#[cfg(feature = "luau")]
pub const LUAU_CONSTANT_NIL: u8 = 0;
#[cfg(feature = "luau")]
pub const LUAU_CONSTANT_BOOLEAN: u8 = 1;
#[cfg(feature = "luau")]
pub const LUAU_CONSTANT_NUMBER: u8 = 2;
#[cfg(feature = "luau")]
pub const LUAU_CONSTANT_STRING: u8 = 3;
#[cfg(feature = "luau")]
pub const LUAU_CONSTANT_IMPORT: u8 = 4;
#[cfg(feature = "luau")]
pub const LUAU_CONSTANT_TABLE: u8 = 5;
#[cfg(feature = "luau")]
pub const LUAU_CONSTANT_CLOSURE: u8 = 6;
#[cfg(feature = "luau")]
pub const LUAU_CONSTANT_VECTOR: u8 = 7;

#[derive(Clone, Debug, Default)]
pub enum Constant {
    #[default]
    Nil,
    Bool(bool),
    Number(f64),
    String(RawLuaString),

    #[cfg(feature = "luau")]
    Vector(f32, f32, f32, f32),
    #[cfg(feature = "luau")]
    Closure(u32),
    #[cfg(feature = "luau")]
    Import(i32),
    #[cfg(feature = "luau")]
    Table(u32, Vec<u32>),
}

// TODO: hacky code, maybe use traits?
impl Constant {
    pub fn kind(&self) -> u8 {
        match self {
            Constant::Nil => LUA_CONSTANT_NIL,
            Constant::Bool(_) => LUA_CONSTANT_BOOLEAN,
            Constant::Number(_) => LUA_CONSTANT_NUMBER,
            Constant::String(_) => LUA_CONSTANT_STRING,

            _ => unreachable!(),
        }
    }

    #[cfg(feature = "luau")]
    pub fn kind_luau(&self) -> u8 {
        match self {
            Constant::Nil => LUAU_CONSTANT_NIL,
            Constant::Bool(_) => LUAU_CONSTANT_BOOLEAN,
            Constant::Number(_) => LUAU_CONSTANT_NUMBER,
            Constant::String(_) => LUAU_CONSTANT_STRING,
            Constant::Vector(_, _, _, _) => LUAU_CONSTANT_VECTOR,
            Constant::Closure(_) => LUAU_CONSTANT_CLOSURE,
            Constant::Import(_) => LUAU_CONSTANT_IMPORT,
            Constant::Table(_, _) => LUAU_CONSTANT_TABLE,
        }
    }
}
