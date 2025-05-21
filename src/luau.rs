use crate::buffer::Buffer;
use crate::opcode::Instruction;
use crate::{Constant, LocalVariable, Proto, RawLuaString, constant};

const LBC_TYPE_TAGGED_USERDATA_END: u8 = 64 + 32;
const LBC_TYPE_TAGGED_USERDATA_BASE: u8 = 64;

#[derive(Default)]
pub struct LuaBytecode {
    pub version: u8,
    pub types_version: u8,

    pub userdata_type_map: Vec<u32>,

    pub protos: Vec<Proto>,
    pub strings: Vec<RawLuaString>,

    pub main_proto_id: u32,
}

impl LuaBytecode {
    pub fn from(data: &[u8]) -> Result<LuaBytecode, String> {
        let mut bytecode = LuaBytecode::default();
        let mut buffer = Buffer::new(data.to_vec());

        bytecode.version = buffer.read::<u8>();
        if bytecode.version == 0 {
            let mut bytes = Vec::new();
            loop {
                let character = buffer.read::<u8>();
                bytes.push(character);

                if character == '\0' as u8 {
                    break;
                }
            }

            let error = std::ffi::CStr::from_bytes_with_nul(bytes.as_slice()).unwrap();
            return Err(format!(
                "Error message in bytecode: {}",
                error.to_str().unwrap()
            ));
        } else if bytecode.version < 4 || bytecode.version > 6 {
            return Err("Bytecode version mismatch".into());
        }

        bytecode.types_version = if bytecode.version >= 4 {
            buffer.read::<u8>()
        } else {
            0
        };

        // read string table
        let string_count = buffer.read_variant();
        for _ in 0..string_count {
            bytecode.strings.push(buffer.read_string());
        }

        let userdata_type_limit = LBC_TYPE_TAGGED_USERDATA_END - LBC_TYPE_TAGGED_USERDATA_BASE;
        bytecode
            .userdata_type_map
            .reserve(userdata_type_limit.into());

        // userdata type remapping table
        if bytecode.types_version == 3 {
            let mut index = buffer.read::<u8>();
            while index != 0 {
                let reference = buffer.read_variant();
                if index - 1 < userdata_type_limit {
                    bytecode
                        .userdata_type_map
                        .insert((index - 1).into(), reference);
                }

                index = buffer.read::<u8>();
            }
        }

        // read proto table
        let proto_count = buffer.read_variant();
        for i in 0..proto_count {
            let proto = bytecode.parse_proto(i as u32, &mut buffer);
            bytecode.protos.push(proto);
        }

        bytecode.main_proto_id = buffer.read_variant();
        Ok(bytecode)
    }

    fn parse_proto(&self, index: u32, buffer: &mut Buffer) -> Proto {
        let mut proto = Proto::default();

        proto.bytecode_id = index;

        proto.max_stack_size = buffer.read::<u8>();
        proto.parameter_count = buffer.read::<u8>();
        proto.upvalue_count = buffer.read::<u8>();
        proto.is_vararg = buffer.read::<bool>();

        if self.version >= 4 {
            proto.flags = buffer.read::<u8>();
            if self.types_version > 0 {
                let types_size = buffer.read_variant();
                if types_size != 0 {
                    let types = buffer.read::<u8>();
                    if self.types_version == 2 || self.types_version == 3 {
                        proto.type_info.push(types);
                    }

                    if self.types_version == 3 {
                        proto.remap_userdata_types(&self.userdata_type_map);
                    }
                }

                buffer.advance(types_size as u64);
            }
        }

        let code_size = buffer.read_variant();
        for _ in 0..code_size {
            let instruction = buffer.read::<u32>();
            let instruction = Instruction::from_bytes(&instruction.to_le_bytes());
            proto.instructions.push(instruction);
        }

        let constant_count = buffer.read_variant();
        for _ in 0..constant_count {
            let kind = buffer.read::<u8>();

            let constant = match kind {
                constant::LUAU_CONSTANT_NIL => Constant::Nil,
                constant::LUAU_CONSTANT_BOOLEAN => Constant::Bool(buffer.read::<bool>()),

                constant::LUAU_CONSTANT_NUMBER => Constant::Number(buffer.read::<f64>()),

                constant::LUAU_CONSTANT_STRING => {
                    Constant::String(self.string_from_reference(buffer).unwrap())
                }

                constant::LUAU_CONSTANT_IMPORT => Constant::Import(buffer.read::<i32>()),

                constant::LUAU_CONSTANT_TABLE => {
                    let length = buffer.read_variant();

                    let mut keys = vec![];
                    for _ in 0..length {
                        let key = buffer.read_variant();
                        keys.push(key);
                    }

                    Constant::Table(length, keys)
                }

                constant::LUAU_CONSTANT_CLOSURE => {
                    let proto_id = buffer.read_variant();
                    Constant::Closure(proto_id)
                }

                constant::LUAU_CONSTANT_VECTOR => Constant::Vector(
                    buffer.read::<f32>(),
                    buffer.read::<f32>(),
                    buffer.read::<f32>(),
                    buffer.read::<f32>(),
                ),

                _ => unreachable!(),
            };

            proto.constants.push(constant);
        }

        let children = buffer.read_variant();
        for _ in 0..children {
            let proto_id = buffer.read_variant();
            proto.protos.push(proto_id);
        }

        proto.line_defined = buffer.read_variant();
        proto.name = self.string_from_reference(buffer);

        let has_lineinfo = buffer.read::<u8>() != 0;
        if has_lineinfo {
            proto.linegaplog2 = buffer.read::<u8>();
            let intervals = ((proto.instructions.len() - 1) >> proto.linegaplog2) + 1;

            for _ in 0..proto.instructions.len() {
                let last_offset = buffer.read::<u8>();
                proto.line_info.push(last_offset as u32);
            }

            for _ in 0..intervals {
                let last_line = buffer.read::<i32>();
                proto.absolute_line_info.push(last_line);
            }
        }

        let has_debuginfo = buffer.read::<bool>();
        if has_debuginfo {
            let locvar_count = buffer.read_variant();
            for _ in 0..locvar_count {
                proto.locals.push(LocalVariable {
                    name: self.string_from_reference(buffer).unwrap(),
                    start_pc: buffer.read_variant(),
                    end_pc: buffer.read_variant(),
                    register: buffer.read::<u8>(),
                });
            }

            let upvalue_count = buffer.read_variant();
            assert_eq!(upvalue_count as u8, proto.upvalue_count);

            for _ in 0..upvalue_count {
                proto
                    .upvalues
                    .push(self.string_from_reference(buffer).unwrap());
            }
        }

        proto
    }

    pub fn write(&self) -> Vec<u8> {
        let mut buffer = Buffer::new(Vec::new());

        buffer.write(self.version);
        buffer.write(self.types_version);

        // write string table
        buffer.write_variant(self.strings.len() as u32);
        for string in self.strings.iter() {
            buffer.write_string(string.clone());
        }

        // write userdata type remapping table
        if self.types_version == 3 {
            for (index, name_reference) in self.userdata_type_map.iter().enumerate() {
                buffer.write::<u8>(index as u8 + 1);
                buffer.write_variant(*name_reference);
            }
            buffer.write::<u8>(0);
        }

        // write proto table
        buffer.write_variant(self.protos.len() as u32);
        for i in 0..self.protos.len() {
            self.write_proto(i as u32, &mut buffer);
        }

        buffer.write_variant(self.main_proto_id);

        buffer.set_position(0);
        buffer.read_all()
    }

    fn write_proto(&self, index: u32, buffer: &mut Buffer) {
        let proto = &self.protos[index as usize];

        buffer.write(proto.max_stack_size);
        buffer.write(proto.parameter_count);
        buffer.write(proto.upvalue_count as u8);
        buffer.write(proto.is_vararg as u8);

        if self.version >= 4 {
            buffer.write::<u8>(proto.flags);
            if self.types_version > 0 {
                buffer.write_variant(proto.type_info.len() as u32);

                if proto.type_info.len() > 0 {
                    for i in 0..proto.type_info.len() {
                        buffer.write::<u8>(proto.type_info[i]);
                    }
                }
            }
        }

        buffer.write_variant(proto.instructions.len() as u32);
        for instruction in proto.instructions.iter() {
            buffer.write::<i32>(instruction.0 as i32);
        }

        buffer.write_variant(proto.constants.len() as u32);
        for constant in proto.constants.iter() {
            buffer.write::<u8>(constant.kind_luau());

            match constant {
                Constant::Nil => (),

                Constant::Bool(value) => buffer.write(*value),
                Constant::Number(value) => buffer.write(*value),

                Constant::String(value) => {
                    let reference = self.string_reference(value.clone());
                    buffer.write_variant(reference);
                }

                Constant::Table(length, keys) => {
                    buffer.write_variant(*length);
                    for key in keys {
                        buffer.write_variant(*key);
                    }
                }

                Constant::Closure(proto_id) => {
                    buffer.write_variant(*proto_id);
                }

                Constant::Vector(x, y, z, w) => {
                    buffer.write(x);
                    buffer.write(y);
                    buffer.write(z);
                    buffer.write(w);
                }

                Constant::Import(value) => buffer.write(*value),
            }
        }

        buffer.write_variant(proto.protos.len() as u32);
        for proto in proto.protos.iter() {
            buffer.write_variant(*proto);
        }

        buffer.write_variant(proto.line_defined as u32);
        if proto.name.is_some() {
            let name_reference = self.string_reference(proto.name.as_ref().unwrap().clone());
            buffer.write_variant(name_reference);
        } else {
            buffer.write_variant(0);
        }

        let has_lines = proto.line_info.len() > 0 || proto.absolute_line_info.len() > 0;
        if has_lines {
            buffer.write::<u8>(1);
            buffer.write::<u8>(proto.linegaplog2);

            for i in 0..proto.instructions.len() {
                buffer.write::<u8>(proto.line_info[i] as u8);
            }

            for i in 0..proto.absolute_line_info.len() {
                buffer.write::<i32>(proto.absolute_line_info[i]);
            }
        } else {
            buffer.write::<u8>(0);
        }

        let has_debug = proto.locals.len() > 0 || proto.upvalues.len() > 0;
        if has_debug {
            buffer.write::<u8>(1);

            buffer.write_variant(proto.locals.len() as u32);
            for local in proto.locals.iter() {
                let name_reference = self.string_reference(local.name.clone());
                buffer.write_variant(name_reference);
                buffer.write_variant(local.start_pc);
                buffer.write_variant(local.end_pc);
                buffer.write(local.register);
            }

            buffer.write_variant(proto.upvalues.len() as u32);
            for upvalue in proto.upvalues.iter() {
                let reference = self.string_reference(upvalue.clone());
                buffer.write_variant(reference);
            }
        } else {
            buffer.write::<u8>(0);
        }
    }

    fn string_from_reference(&self, buffer: &mut Buffer) -> Option<RawLuaString> {
        let id = buffer.read_variant();
        if id == 0 {
            return None;
        }

        Some(self.strings.get(id as usize - 1).unwrap().clone())
    }

    fn string_reference(&self, string: RawLuaString) -> u32 {
        self.strings.iter().position(|s| s == &string).unwrap() as u32 + 1
    }
}

trait LuauProto {
    fn remap_userdata_types(&mut self, userdata_type_map: &Vec<u32>);
}

impl LuauProto for Proto {
    fn remap_userdata_types(&mut self, userdata_type_map: &Vec<u32>) {
        let buffer = &mut Buffer::new(self.type_info.clone());

        let count = LBC_TYPE_TAGGED_USERDATA_END - LBC_TYPE_TAGGED_USERDATA_BASE;

        let type_size = buffer.read_variant();
        let upvalue_count = buffer.read_variant();
        let local_count = buffer.read_variant();

        if type_size != 0 {
            for i in 2..type_size {
                let _type = buffer.read::<u8>();
                let index = _type - LBC_TYPE_TAGGED_USERDATA_BASE;
                if index < count {
                    buffer.write::<u8>((userdata_type_map[index as usize]) as u8);
                }
            }

            buffer.advance(type_size as u64);
        }

        if upvalue_count != 0 {
            for i in 0..upvalue_count {
                let _type = buffer.read::<u8>();
                let index = _type - LBC_TYPE_TAGGED_USERDATA_BASE;
                if index < count {
                    buffer.write::<u8>((userdata_type_map[index as usize]) as u8);
                }
            }

            buffer.advance(upvalue_count as u64);
        }

        if local_count != 0 {
            for i in 0..local_count {
                let _type = buffer.read::<u8>();
                let index = _type - LBC_TYPE_TAGGED_USERDATA_BASE;
                if index < count {
                    buffer.write::<u8>((userdata_type_map[index as usize]) as u8);
                }

                buffer.advance(2);
                buffer.read_variant();
                buffer.read_variant();
            }
        }
    }
}

trait Variant {
    fn read_variant(&mut self) -> u32;
    fn write_variant(&mut self, value: u32);

    fn read_string(&mut self) -> RawLuaString;
    fn write_string(&mut self, string: RawLuaString);
}

impl Variant for Buffer {
    fn read_variant(&mut self) -> u32 {
        let mut value: u32 = 0;
        let mut shift: u32 = 0;

        loop {
            let byte = self.read::<u8>() as u8;
            value |= (byte as u32 & 127) << shift;
            shift += 7;

            if (byte & 128) == 0 {
                break;
            }
        }

        value
    }

    fn write_variant(&mut self, mut value: u32) {
        loop {
            self.write::<u8>(((value & 127) | (((value > 127) as u32) << 7)) as u8);
            value >>= 7;
            if value == 0 {
                break;
            }
        }
    }

    fn read_string(&mut self) -> RawLuaString {
        let mut bytes = Vec::new();

        let length = self.read_variant();
        for _ in 0..length {
            bytes.push(self.read::<u8>());
        }

        bytes
    }

    fn write_string(&mut self, string: RawLuaString) {
        self.write_variant(string.len() as u32);
        for byte in string {
            self.write(byte);
        }
    }
}
