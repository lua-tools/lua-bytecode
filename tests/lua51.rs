#[cfg(feature = "lua51")]
use lua_bytecode::{Bytecode, lua51::LuaBytecode};

use std::process::Command;

#[cfg(feature = "lua51")]
fn compile(name: &str) -> Vec<u8> {
    std::fs::create_dir_all("tests/cache").unwrap();
    let result = Command::new("sh")
        .arg("-c")
        .arg(format!("(luac5.1 -o tests/cache/bytecode.out tests/lua51/{}.lua)", name))
        .output()
        .unwrap();

    assert!(result.status.success());
    std::fs::read("tests/cache/bytecode.out").unwrap()
}

#[test]
#[cfg(feature = "lua51")]
fn number() {
    let mut bytecode = <Bytecode as LuaBytecode>::new();
    bytecode.parse(compile("number").as_slice());

    let main_proto = &bytecode.protos[bytecode.main_proto_id as usize];

    assert_eq!(bytecode.protos.len(), 1);
    assert_eq!(main_proto.locals.len(), 3);
    assert_eq!(main_proto.constants.len(), 2);

    let data = bytecode.write();
    let mut bytecode = <Bytecode as LuaBytecode>::new();
    bytecode.parse(data.as_slice());

    let main_proto = &bytecode.protos[bytecode.main_proto_id as usize];

    assert_eq!(bytecode.protos.len(), 1);
    assert_eq!(main_proto.locals.len(), 3);
    assert_eq!(main_proto.constants.len(), 2);
}
