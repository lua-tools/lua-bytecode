#![cfg(feature = "luau")]

use lua_bytecode::{
    luau::LuaBytecode,
    opcode::{Instruction, LuauInstruction, LuauOpcode, Opcode},
};

use std::process::Command;

fn compile(name: &str) -> Vec<u8> {
    let result = Command::new("luau-compile")
        .arg("--binary")
        .arg(format!("tests/luau/{}.lua", name))
        .output()
        .unwrap();

    assert!(result.status.success());
    result.stdout
}

#[test]
fn number() {
    let bytecode = LuaBytecode::from(compile("number").as_slice()).unwrap();
    let main_proto = &bytecode.protos[bytecode.main_proto_id as usize];

    assert_eq!(bytecode.protos.len(), 1);
    assert_eq!(main_proto.locals.len(), 0);
    assert_eq!(main_proto.constants.len(), 2);

    let data = bytecode.write();

    let bytecode = LuaBytecode::from(data.as_slice()).unwrap();
    let main_proto = &bytecode.protos[bytecode.main_proto_id as usize];

    assert_eq!(bytecode.protos.len(), 1);
    assert_eq!(main_proto.locals.len(), 0);
    assert_eq!(main_proto.constants.len(), 2);
}

#[test]
fn map_iter() {
    let bytecode = LuaBytecode::from(compile("map_iter").as_slice()).unwrap();
    let main_proto = &bytecode.protos[bytecode.main_proto_id as usize];

    assert_eq!(bytecode.protos.len(), 1);
    assert_eq!(main_proto.locals.len(), 0);
    assert_eq!(main_proto.constants.len(), 7);

    let data = bytecode.write();

    let bytecode = LuaBytecode::from(data.as_slice()).unwrap();
    let main_proto = &bytecode.protos[bytecode.main_proto_id as usize];

    assert_eq!(bytecode.protos.len(), 1);
    assert_eq!(main_proto.locals.len(), 0);
    assert_eq!(main_proto.constants.len(), 7);
}

#[test]
fn instruction() {
    match Instruction(0).opcode() {
        lua_bytecode::opcode::Opcode::LuauOpcode(op) => {
            assert_eq!(op, LuauOpcode::Nop);
        }

        _ => unreachable!(),
    }

    match Instruction(82).opcode() {
        lua_bytecode::opcode::Opcode::LuauOpcode(op) => {
            assert_eq!(op, LuauOpcode::IDivK);
        }

        _ => unreachable!(),
    }

    let instruction = Instruction::from_abc(Opcode::LuauOpcode(LuauOpcode::Call), 0, 1 + 1, 1);
    match instruction.opcode() {
        Opcode::LuauOpcode(op) => {
            assert_eq!(op, LuauOpcode::Call);

            assert_eq!(instruction.a(), 0); // base
            assert_eq!(instruction.b(), 2); // parameters + 1
            assert_eq!(instruction.c(), 1); // ?
        }
        _ => unreachable!(),
    }

    let instruction = Instruction::from_ad(Opcode::LuauOpcode(LuauOpcode::LoadK), 0, 0);
    match instruction.opcode() {
        Opcode::LuauOpcode(op) => {
            assert_eq!(op, LuauOpcode::LoadK);

            assert_eq!(instruction.a(), 0); // target
            assert_eq!(instruction.d(), 0); // constant
        }
        _ => unreachable!(),
    }
}
