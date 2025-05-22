#![cfg(feature = "lua51")]

use lua_bytecode::{
    Bytecode,
    lua51::LuaBytecode,
    opcode::{LuaOpMode, Opcode},
};

use std::process::Command;

fn compile(name: &str) -> Vec<u8> {
    std::fs::create_dir_all("tests/cache").unwrap();
    let result = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "(luac5.1 -o tests/cache/bytecode.out tests/lua51/{}.lua)",
            name
        ))
        .output()
        .unwrap();

    assert!(result.status.success());
    std::fs::read("tests/cache/bytecode.out").unwrap()
}

#[test]
fn number() {
    let mut bytecode = <Bytecode as LuaBytecode>::from(compile("number").as_slice()).unwrap();
    let main_proto = &bytecode.protos[bytecode.main_proto_id as usize];

    assert_eq!(bytecode.protos.len(), 1);
    assert_eq!(main_proto.locals.len(), 3);
    assert_eq!(main_proto.constants.len(), 2);

    let data = bytecode.write();

    let bytecode = <Bytecode as LuaBytecode>::from(data.as_slice()).unwrap();
    let main_proto = &bytecode.protos[bytecode.main_proto_id as usize];

    assert_eq!(bytecode.protos.len(), 1);
    assert_eq!(main_proto.locals.len(), 3);
    assert_eq!(main_proto.constants.len(), 2);
}

#[test]
fn instruction() {
    use lua_bytecode::opcode::{Instruction, LuaInstruction, LuaOpcode};

    match Instruction(0).opcode() {
        lua_bytecode::opcode::Opcode::LuaOpcode(op) => {
            assert_eq!(op, LuaOpcode::Move);
        }

        _ => unreachable!(),
    }

    match Instruction(1).opcode() {
        lua_bytecode::opcode::Opcode::LuaOpcode(op) => {
            assert_eq!(op, LuaOpcode::LoadK);
        }

        _ => unreachable!(),
    }

    match Instruction(37).opcode() {
        lua_bytecode::opcode::Opcode::LuaOpcode(op) => {
            assert_eq!(op, LuaOpcode::Vararg);
        }

        _ => unreachable!(),
    }

    let instruction = Instruction::from_abc(Opcode::LuaOpcode(LuaOpcode::Call), 0, 1 + 1, 1);
    match instruction.opcode() {
        Opcode::LuaOpcode(op) => {
            assert_eq!(op, LuaOpcode::Call);
            assert_eq!(op.mode(), LuaOpMode::IABC);

            assert_eq!(instruction.a(), 0); // base
            assert_eq!(instruction.b(), 2); // parameters + 1
            assert_eq!(instruction.c(), 1); // ?
        }
        _ => unreachable!(),
    }

    let instruction = Instruction::from_abx(Opcode::LuaOpcode(LuaOpcode::GetGlobal), 0, 100);
    match instruction.opcode() {
        Opcode::LuaOpcode(op) => {
            assert_eq!(op, LuaOpcode::GetGlobal);
            assert_eq!(op.mode(), LuaOpMode::IABx);

            assert_eq!(instruction.a(), 0); // base
            assert_eq!(instruction.bx(), 100); // global
        }
        _ => unreachable!(),
    }
}
