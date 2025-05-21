#[cfg(feature = "luau")]
use lua_bytecode::luau::LuaBytecode;

use std::process::Command;

#[cfg(feature = "luau")]
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
#[cfg(feature = "luau")]
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
#[cfg(feature = "luau")]
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
