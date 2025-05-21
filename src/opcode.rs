#[cfg(feature = "lua51")]
const LUA_OP_SIZE: u32 = 6;
#[cfg(feature = "lua51")]
const LUA_OP_POSITION: u32 = 0;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg(feature = "lua51")]
pub enum LuaOpcode {
    Move,
    LoadK,
    LoadBool,
    LoadNil,
    GetUpval,

    GetGlobal,
    GetTable,

    SetGlobal,
    SetUpval,
    SetTable,

    NewTable,

    Self_,

    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Unm,
    Not,
    Len,

    Concat,

    Jmp,

    Eq,
    Lt,
    Le,

    Test,
    TestSet,

    Call,
    TailCall,
    Return,

    ForLoop,
    ForPrep,

    TForLoop,
    SetList,

    Close,
    Closure,

    Vararg,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg(feature = "luau")]
pub enum LuauOpcode {
    Nop,
    Break,
    LoadNil,
    LoadB,
    LoadN,
    LoadK,
    Move,
    GetGlobal,
    SetGlobal,
    GetUpval,
    SetUpval,
    CloseUpvals,
    GetImport,
    GetTable,
    SetTable,
    GetTableKs,
    SetTableKs,
    GetTableN,
    SetTableN,
    NewClosure,
    NameCall,
    Call,
    Return,
    Jump,
    JumpBack,
    JumpIf,
    JumpIfNot,
    JumpIfEq,
    JumpIfLe,
    JumpIfLt,
    JumpIfNotEq,
    JumpIfNotLe,
    JumpIfNotLt,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    AddK,
    SubK,
    MulK,
    DivK,
    ModK,
    PowK,
    And,
    Or,
    AndK,
    OrK,
    Concat,
    Not,
    Minus,
    Length,
    NewTable,
    DupTable,
    SetList,
    ForNPrep,
    ForNLoop,
    ForGLoop,
    ForGPrepInext,
    FastCall3,
    ForGPrepNext,
    NativeCall,
    GetVarargs,
    DupClosure,
    PrepVarargs,
    LoadKx,
    JumpX,
    FastCall,
    Coverage,
    Capture,
    SubRk,
    DivRk,
    FastCall1,
    FastCall2,
    FastCall2K,
    ForGPrep,
    JumpXeqkNil,
    JumpXeqkB,
    JumpXeqkN,
    JumpXeqkS,
    IDiv,
    IDivK,
}

#[cfg(feature = "lua51")]
impl LuaOpcode {
    pub(crate) fn index(op: u8) -> LuaOpcode {
        unsafe { std::mem::transmute(op) }
    }
}

#[cfg(feature = "luau")]
impl LuauOpcode {
    pub(crate) fn index(op: u8) -> LuauOpcode {
        unsafe { std::mem::transmute(op) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpCode {
    #[cfg(feature = "lua51")]
    LuaOpcode(LuaOpcode),
    #[cfg(feature = "luau")]
    LuauOpcode(LuauOpcode),
}

pub struct Instruction(pub u32);

#[cfg(feature = "lua51")]
pub trait LuaInstruction {
    fn opcode(&self) -> OpCode;
    /* creates a mask with `n' 0 bits at position `p' */
    fn mask_0(&self, n: u32, p: u32) -> u32;
    /* creates a mask with `n' 1 bits at position `p' */
    fn mask_1(&self, n: u32, p: u32) -> u32;
}

#[cfg(feature = "luau")]
pub trait LuauInstruction {
    fn opcode(&self) -> OpCode;
}

#[cfg(feature = "lua51")]
impl LuaInstruction for Instruction {
    fn opcode(&self) -> OpCode {
        let op = ((self.0 >> LUA_OP_POSITION) as u8) & self.mask_1(LUA_OP_SIZE, 0) as u8;
        OpCode::LuaOpcode(LuaOpcode::index(op))
    }

    fn mask_1(&self, n: u32, p: u32) -> u32 {
        ((1u32 << n) - 1) << p
    }

    fn mask_0(&self, n: u32, p: u32) -> u32 {
        !self.mask_1(n, p)
    }
}

#[cfg(feature = "luau")]
impl LuauInstruction for Instruction {
    fn opcode(&self) -> OpCode {
        let op = (self.0 & 0xff) as u8;
        OpCode::LuauOpcode(LuauOpcode::index(op))
    }
}

impl Instruction {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Instruction(u32::from_le_bytes(bytes.try_into().unwrap()))
    }
}
