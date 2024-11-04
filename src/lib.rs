mod buffer;

#[cfg(feature = "luau")]
pub mod luau;
#[cfg(feature = "lua51")]
pub mod lua51;

enum Format {
    Lua51,
    Lua52,
    Lua53,
    Lua54,
    LuaJit,
    Luau
}

struct Header {
    pub version: u8,
    pub format: u8,

    pub is_big_endian: bool,
    pub is_number_integral: bool,

    pub int_size: u8,
    pub size_t_size: u8,
    pub number_size: u8,
    pub instruction_size: u8,

    pub luajit_flags: u8
}

pub struct LocalVariable {
    name: String,
    start_pc: u32,
    end_pc: u32,

    #[cfg(feature = "luau")]
    register: u8
}

const LUA_CONSTANT_NIL: u8 = 0;
const LUA_CONSTANT_BOOLEAN: u8 = 1;
const LUA_CONSTANT_NUMBER: u8 = 3;
const LUA_CONSTANT_STRING: u8 = 4;

pub struct Constant {
    kind: u8,
    value: Vec<u8>
}

impl Constant {
     fn new() -> Self {
         Constant {
             kind: 0,
             value: Vec::new()
         }
     }
}

pub struct Instruction(pub u32);

impl Instruction {
    fn from_bytes(bytes: &[u8]) -> Self {
        Instruction(u32::from_le_bytes(bytes.try_into().unwrap()))
    }
}


#[derive(Default)]
pub struct Proto {
    pub bytecode_id: u32,

    pub max_stack_size: u8,
    pub parameters_count: u8,
    pub is_vararg: bool,

    pub flags: u8,
    pub type_info: Vec<u8>,

    pub line_defined: u32,
    pub last_line_defined: u32,

    pub name: Option<String>,
    pub line_info: Vec<u8>,
    pub absolute_line_info: Vec<i32>,
    pub linegaplog2: u8,

    pub protos: Vec<u32>,
    pub locals: Vec<LocalVariable>,
    pub upvalues: Vec<String>,
    pub constants: Vec<Constant>,
    pub instructions: Vec<Instruction>
}

impl Proto {
    fn new() -> Self {
        Default::default()
    }
}

pub struct Bytecode {
    pub header: Header,
    pub protos: Vec<Proto>,
    pub main_proto_id: u32
}
