#[cfg(feature = "lua51")]
const LUA_OP_SIZE: u32 = 6;
#[cfg(feature = "lua51")]
const LUA_A_SIZE: u32 = 8;
#[cfg(feature = "lua51")]
const LUA_B_SIZE: u32 = 9;
#[cfg(feature = "lua51")]
const LUA_C_SIZE: u32 = 9;
#[cfg(feature = "lua51")]
const LUA_BX_SIZE: u32 = LUA_B_SIZE + LUA_C_SIZE;

#[cfg(feature = "lua51")]
const LUA_OP_POSITION: u32 = 0;
#[cfg(feature = "lua51")]
const LUA_A_POSITION: u32 = LUA_OP_SIZE;
#[cfg(feature = "lua51")]
const LUA_C_POSITION: u32 = LUA_A_POSITION + LUA_A_SIZE;
#[cfg(feature = "lua51")]
const LUA_B_POSITION: u32 = LUA_C_POSITION + LUA_C_SIZE;
#[cfg(feature = "lua51")]
const LUA_BX_POSITION: u32 = LUA_C_POSITION;

#[cfg(feature = "lua51")]
pub const MAX_ARG_BX: u32 = (1 << LUA_BX_SIZE) - 1;
#[cfg(feature = "lua51")]
pub const MAX_ARG_SBX: i32 = (MAX_ARG_BX as i32) >> 1;

#[derive(Debug, PartialEq, Eq)]
pub enum LuaOpMode {
    IA,
    IAB,
    IAC,
    IABC,

    IAx,
    IABx,
    IAsBx,
}

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
    pub fn index(op: u8) -> LuaOpcode {
        unsafe { std::mem::transmute(op) }
    }

    pub fn mode(&self) -> LuaOpMode {
        match self {
            LuaOpcode::Move => LuaOpMode::IABC,
            LuaOpcode::LoadK => LuaOpMode::IABx,
            LuaOpcode::LoadBool => LuaOpMode::IABC,
            LuaOpcode::LoadNil => LuaOpMode::IAB,

            LuaOpcode::GetUpval | LuaOpcode::SetUpval => LuaOpMode::IAB,
            LuaOpcode::GetGlobal | LuaOpcode::SetGlobal => LuaOpMode::IABx,
            LuaOpcode::GetTable | LuaOpcode::SetTable | LuaOpcode::NewTable => LuaOpMode::IABC,
            LuaOpcode::Self_ => LuaOpMode::IABC,

            LuaOpcode::Add
            | LuaOpcode::Sub
            | LuaOpcode::Mul
            | LuaOpcode::Div
            | LuaOpcode::Mod
            | LuaOpcode::Pow => LuaOpMode::IABC,

            LuaOpcode::Unm | LuaOpcode::Not | LuaOpcode::Len => LuaOpMode::IAB,

            LuaOpcode::Concat => LuaOpMode::IABC,
            LuaOpcode::Jmp => LuaOpMode::IAsBx, // sBx?

            LuaOpcode::Eq | LuaOpcode::Lt | LuaOpcode::Le => LuaOpMode::IABC,

            LuaOpcode::Test => LuaOpMode::IAC,
            LuaOpcode::TestSet => LuaOpMode::IABC,

            LuaOpcode::Call | LuaOpcode::TailCall => LuaOpMode::IABC,
            LuaOpcode::Return => LuaOpMode::IAB,

            LuaOpcode::ForLoop => LuaOpMode::IAsBx,
            LuaOpcode::ForPrep => LuaOpMode::IAsBx,
            LuaOpcode::TForLoop => LuaOpMode::IAC,
            LuaOpcode::SetList => LuaOpMode::IABC,

            LuaOpcode::Close => LuaOpMode::IA,
            LuaOpcode::Closure => LuaOpMode::IABx,

            LuaOpcode::Vararg => LuaOpMode::IAB,
        }
    }
}

#[cfg(feature = "luau")]
impl LuauOpcode {
    pub fn index(op: u8) -> LuauOpcode {
        unsafe { std::mem::transmute(op) }
    }

    pub fn length(&self) -> u8 {
        match self {
            LuauOpcode::GetGlobal
            | LuauOpcode::SetGlobal
            | LuauOpcode::GetImport
            | LuauOpcode::GetTableKs
            | LuauOpcode::SetTableKs
            | LuauOpcode::NameCall
            | LuauOpcode::JumpIfEq
            | LuauOpcode::JumpIfLt
            | LuauOpcode::JumpIfLe
            | LuauOpcode::JumpIfNotEq
            | LuauOpcode::JumpIfNotLt
            | LuauOpcode::JumpIfNotLe
            | LuauOpcode::NewTable
            | LuauOpcode::SetList
            | LuauOpcode::ForGLoop
            | LuauOpcode::FastCall3
            | LuauOpcode::LoadKx
            | LuauOpcode::FastCall2
            | LuauOpcode::FastCall2K
            | LuauOpcode::JumpXeqkB
            | LuauOpcode::JumpXeqkN
            | LuauOpcode::JumpXeqkS => 2,
            _ => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opcode {
    #[cfg(feature = "lua51")]
    LuaOpcode(LuaOpcode),
    #[cfg(feature = "luau")]
    LuauOpcode(LuauOpcode),
}

pub struct Instruction(pub u32);

#[cfg(feature = "lua51")]
pub trait LuaInstruction {
    fn opcode(&self) -> Opcode;
    fn from_abc(op: Opcode, a: u32, b: u32, c: u32) -> Self;
    fn from_abx(opcode: Opcode, a: u32, bx: u32) -> Self;

    fn get(&self, size: u32, position: u32) -> u32;
    fn a(&self) -> u32;
    fn b(&self) -> u32;
    fn c(&self) -> u32;
    fn bx(&self) -> u32;
    fn sbx(&self) -> i32;

    fn set(&mut self, value: u32, position: u32, size: u32) -> &mut Self;
    fn set_a(&mut self, a: u32) -> &mut Self;
    fn set_b(&mut self, b: u32) -> &mut Self;
    fn set_c(&mut self, c: u32) -> &mut Self;
    fn set_bx(&mut self, bx: u32) -> &mut Self;
    fn set_sbx(&mut self, sbx: i32) -> &mut Self;

    /* creates a mask with `n' 0 bits at position `p' */
    fn mask_0(&self, n: u32, p: u32) -> u32;
    /* creates a mask with `n' 1 bits at position `p' */
    fn mask_1(&self, n: u32, p: u32) -> u32;
}

#[cfg(feature = "luau")]
pub trait LuauInstruction {
    fn opcode(&self) -> Opcode;

    fn from_abc(op: Opcode, a: u32, b: u32, c: u32) -> Self;
    fn from_ad(opcode: Opcode, a: u32, d: u32) -> Self;
    fn from_e(opcode: Opcode, e: u32) -> Self;

    fn a(&self) -> u32;
    fn b(&self) -> u32;
    fn c(&self) -> u32;
    fn d(&self) -> u32;
    fn e(&self) -> u32;
}

#[cfg(feature = "lua51")]
impl LuaInstruction for Instruction {
    fn opcode(&self) -> Opcode {
        let op = ((self.0 >> LUA_OP_POSITION) as u8) & self.mask_1(LUA_OP_SIZE, 0) as u8;
        Opcode::LuaOpcode(LuaOpcode::index(op))
    }

    fn from_abc(opcode: Opcode, a: u32, b: u32, c: u32) -> Self {
        let mut instruction = match opcode {
            Opcode::LuaOpcode(op) => Instruction(op as u32),
            _ => unreachable!(),
        };

        Instruction(instruction.set_a(a).set_b(b).set_c(c).0)
    }

    fn from_abx(opcode: Opcode, a: u32, bx: u32) -> Self {
        let mut instruction = match opcode {
            Opcode::LuaOpcode(op) => Instruction(op as u32),
            _ => unreachable!(),
        };

        Instruction(instruction.set_a(a).set_bx(bx).0)
    }

    fn a(&self) -> u32 {
        self.get(LUA_A_SIZE, LUA_A_POSITION)
    }

    fn b(&self) -> u32 {
        self.get(LUA_B_SIZE, LUA_B_POSITION)
    }

    fn c(&self) -> u32 {
        self.get(LUA_C_SIZE, LUA_C_POSITION)
    }

    fn bx(&self) -> u32 {
        self.get(LUA_BX_SIZE, LUA_BX_POSITION)
    }

    fn sbx(&self) -> i32 {
        (self.bx() as i32) - MAX_ARG_SBX
    }

    fn set_a(&mut self, a: u32) -> &mut Self {
        self.set(a, LUA_A_SIZE, LUA_A_POSITION)
    }

    fn set_b(&mut self, b: u32) -> &mut Self {
        self.set(b, LUA_B_SIZE, LUA_B_POSITION)
    }

    fn set_c(&mut self, c: u32) -> &mut Self {
        self.set(c, LUA_C_SIZE, LUA_C_POSITION)
    }

    fn set_bx(&mut self, bx: u32) -> &mut Self {
        self.set(bx, LUA_BX_SIZE, LUA_BX_POSITION)
    }

    fn set_sbx(&mut self, bx: i32) -> &mut Self {
        self.set_bx((bx + MAX_ARG_SBX) as u32)
    }

    fn get(&self, size: u32, position: u32) -> u32 {
        (self.0 >> position) & self.mask_1(size, 0)
    }

    fn set(&mut self, value: u32, size: u32, position: u32) -> &mut Self {
        self.0 = (self.mask_1(size, position) & (value << position))
            | (self.0 & self.mask_0(size, position));

        self
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
    fn opcode(&self) -> Opcode {
        let op = (self.0 & 0xff) as u8;
        Opcode::LuauOpcode(LuauOpcode::index(op))
    }

    fn from_abc(opcode: Opcode, a: u32, b: u32, c: u32) -> Self {
        let op = match opcode {
            Opcode::LuauOpcode(op) => op as u32,
            _ => unreachable!(),
        };

        Instruction((op) | (a << 8) | (b << 16) | (c << 24))
    }

    fn from_ad(opcode: Opcode, a: u32, d: u32) -> Self {
        let op = match opcode {
            Opcode::LuauOpcode(op) => op as u32,
            _ => unreachable!(),
        };

        Instruction((op) | (a << 8) | (d << 16))
    }

    fn from_e(opcode: Opcode, e: u32) -> Self {
        let op = match opcode {
            Opcode::LuauOpcode(op) => op as u32,
            _ => unreachable!(),
        };

        Instruction(op | e << 8)
    }

    fn a(&self) -> u32 {
        (self.0 >> 8) & 0xff
    }

    fn b(&self) -> u32 {
        (self.0 >> 16) & 0xff
    }

    fn c(&self) -> u32 {
        (self.0 >> 24) & 0xff
    }

    fn d(&self) -> u32 {
        LuauInstruction::b(self) | LuauInstruction::c(self)
    }

    fn e(&self) -> u32 {
        LuauInstruction::a(self) | LuauInstruction::d(self)
    }
}

impl Instruction {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Instruction(u32::from_le_bytes(bytes.try_into().unwrap()))
    }
}
