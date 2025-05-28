#![allow(dead_code)]

use constant::Constant;

mod buffer;
pub mod constant;
pub mod opcode;

#[cfg(feature = "lua51")]
pub mod lua51;
#[cfg(feature = "luau")]
pub mod luau;

#[cfg(feature = "lua51")]
pub const LUA_MAGIC: u32 = 0x61754c1b;

enum Format {
    Lua51,
    Lua52,
    Lua53,
    Lua54,
    LuaJit,
    Luau,
}

type RawLuaString = Vec<u8>;

#[cfg(feature = "lua51")]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Header {
    pub version: u8,
    pub format: u8,

    pub is_big_endian: bool,

    pub int_size: u8,
    pub size_t_size: u8,
    pub instruction_size: u8,
    pub number_size: u8,

    pub is_number_integral: bool,
    pub luajit_flags: u8,
}

#[cfg(feature = "lua51")]
#[derive(Clone, Debug, Default)]
pub struct Bytecode {
    pub header: Header,
    pub protos: Vec<Proto>,
    pub main_proto_id: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LocalVariable {
    name: RawLuaString,
    start_pc: u32,
    end_pc: u32,

    #[cfg(feature = "luau")]
    register: u8,
}

#[derive(Clone, Debug, Default)]
pub struct Proto {
    #[cfg(feature = "luau")]
    pub bytecode_id: u32,

    pub max_stack_size: u8,
    pub parameter_count: u8,
    pub upvalue_count: u8,
    pub is_vararg: bool,

    pub flags: u8,
    pub type_info: Vec<u8>,

    pub line_defined: u32,
    pub last_line_defined: u32,

    pub name: Option<RawLuaString>,
    pub line_info: Vec<u32>,
    pub absolute_line_info: Vec<i32>,
    pub linegaplog2: u8,

    pub protos: Vec<u32>,
    pub locals: Vec<LocalVariable>,
    pub upvalues: Vec<RawLuaString>,
    pub constants: Vec<Constant>,
    pub instructions: Vec<opcode::Instruction>,
}
