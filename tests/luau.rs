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
    let mut bytecode = LuaBytecode::new();
    bytecode.parse(compile("number").as_slice());

    let main_proto = &bytecode.protos[bytecode.main_proto_id as usize];

    assert_eq!(bytecode.protos.len(), 1);
    assert_eq!(main_proto.locals.len(), 0);
    assert_eq!(main_proto.constants.len(), 2);

    let data = bytecode.write();
    let mut bytecode = LuaBytecode::new();
    bytecode.parse(data.as_slice());

    let main_proto = &bytecode.protos[bytecode.main_proto_id as usize];

    assert_eq!(bytecode.protos.len(), 1);
    assert_eq!(main_proto.locals.len(), 0);
    assert_eq!(main_proto.constants.len(), 2);
}
