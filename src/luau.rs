use crate::{Instruction, Proto, Constant, LocalVariable};
use crate::buffer::Buffer;

const LBC_TYPE_TAGGED_USERDATA_END: u8 = 64 + 32;
const LBC_TYPE_TAGGED_USERDATA_BASE: u8 = 64;

const LUAU_CONSTANT_NIL: u8 = 0;
const LUAU_CONSTANT_BOOLEAN: u8 = 1;
const LUAU_CONSTANT_NUMBER: u8 = 2;
const LUAU_CONSTANT_STRING: u8 = 3;
const LUAU_CONSTANT_IMPORT: u8 = 4;
const LUAU_CONSTANT_TABLE: u8 = 5;
const LUAU_CONSTANT_CLOSURE: u8 = 6;
const LUAU_CONSTANT_VECTOR: u8 = 7;


#[derive(Default)]
pub struct LuaBytecode {
    pub version: u8,
    pub types_version: u8,

    pub userdata_type_map: Vec<u32>,

    pub protos: Vec<Proto>,
    pub strings: Vec<String>,

    pub main_proto_id: u32
}

impl LuaBytecode {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn parse(&mut self, data: &[u8]) {
        let mut buffer = Buffer::new(data.to_vec());

        self.version = buffer.read::<u8>();
        if self.version == 0 {
            eprintln!("Error message in bytecode");
            return
        } else if self.version < 4 || self.version > 6 {
            eprintln!("Unsupported bytecode version");
            return
        }

        self.types_version = if self.version >= 4 { buffer.read::<u8>() } else { 0 };

        // read string table
        let string_count = buffer.read_variant();
        for _ in 0..string_count {
            self.strings.push(buffer.read_string());
        }

        let userdata_type_limit = LBC_TYPE_TAGGED_USERDATA_END - LBC_TYPE_TAGGED_USERDATA_BASE;
        self.userdata_type_map.reserve(userdata_type_limit.into());

        // userdata type remapping table
        if self.types_version == 3 {
            let mut index = buffer.read::<u8>();
            while index != 0 {
                let reference = buffer.read_variant();
                if index - 1 < userdata_type_limit {
                    self.userdata_type_map.insert((index - 1).into(), reference);
                }

                index = buffer.read::<u8>();
            }
        }

        // read proto table
        let proto_count = buffer.read_variant();
        for i in 0..proto_count {
            let proto = self.parse_proto(i as u32, &mut buffer);
            self.protos.push(proto);
        }

        self.main_proto_id = buffer.read_variant();
    }

    fn parse_proto(&self, index: u32, buffer: &mut Buffer) -> Proto {
        let mut proto = Proto::new();

        proto.bytecode_id = index;

        proto.max_stack_size = buffer.read::<u8>();
        proto.parameters_count = buffer.read::<u8>();
        let upvalues_count = buffer.read::<u8>();
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
            let mut constant = Constant::new();
            constant.kind = buffer.read::<u8>();

            match constant.kind {
                LUAU_CONSTANT_NIL => (),
                LUAU_CONSTANT_BOOLEAN => {
                    constant.value = buffer.read::<u8>().to_le_bytes().to_vec();
                },

                LUAU_CONSTANT_NUMBER => {
                    constant.value = buffer.read::<f64>().to_le_bytes().to_vec();
                },

                LUAU_CONSTANT_STRING => {
                    constant.value = self.string_from_reference(buffer).unwrap().as_bytes().to_vec();
                },

                LUAU_CONSTANT_IMPORT => {
                    constant.value = buffer.read::<i32>().to_le_bytes().to_vec();
                },

                LUAU_CONSTANT_TABLE => {
                    let length = buffer.read_variant();
                    constant.value.extend_from_slice(&length.to_le_bytes());

                    for _ in 0..length {
                        let key = buffer.read_variant();
                        constant.value.extend_from_slice(&key.to_le_bytes());
                    }
                },

                LUAU_CONSTANT_CLOSURE => {
                    let proto_id = buffer.read_variant();
                    constant.value = proto_id.to_le_bytes().to_vec();
                },

                LUAU_CONSTANT_VECTOR => {
                    constant.value.extend_from_slice(&buffer.read::<f32>().to_le_bytes());
                    constant.value.extend_from_slice(&buffer.read::<f32>().to_le_bytes());
                    constant.value.extend_from_slice(&buffer.read::<f32>().to_le_bytes());
                    constant.value.extend_from_slice(&buffer.read::<f32>().to_le_bytes());
                }

                _ => unreachable!()
            }
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
                proto.line_info.push(last_offset);
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
                    name: self.string_from_reference(buffer).unwrap().to_string(),
                    start_pc: buffer.read_variant(),
                    end_pc: buffer.read_variant(),
                    register: buffer.read::<u8>()
                });
            }

            let upvalue_count = buffer.read_variant();
            for _ in 0..upvalue_count {
                proto.upvalues.push(self.string_from_reference(buffer).unwrap().to_string());
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
            buffer.write_string(string);
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
        buffer.write(proto.parameters_count);
        buffer.write(proto.upvalues.len() as u8);
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
            buffer.write::<u8>(constant.kind.clone() as u8);

            match constant.kind {
                LUAU_CONSTANT_STRING => {
                    let reference = self.string_reference(&String::from_utf8(constant.value.clone()).unwrap());
                    buffer.write_variant(reference);
                }

                LUAU_CONSTANT_TABLE => {
                    let mut shape = Buffer::new(constant.value.clone());
                    let length = shape.read_variant();

                    buffer.write_variant(length);
                    for _ in 0..length {
                        let key = shape.read_variant();
                        buffer.write_variant(key);
                    }
                }

                LUAU_CONSTANT_CLOSURE => {
                    let proto_id = u32::from_le_bytes(constant.value.clone().as_slice().try_into().unwrap());
                    buffer.write_variant(proto_id);
                }

                _ => {
                    for byte in constant.value.iter() {
                        buffer.write::<u8>(*byte);
                    }
                }
            }
        }

        buffer.write_variant(proto.protos.len() as u32);
        for proto in proto.protos.iter() {
            buffer.write_variant(*proto);
        }

        buffer.write_variant(proto.line_defined as u32);
        if proto.name.is_some() {
            let name_reference = self.string_reference(&proto.name.as_ref().unwrap().to_string());
            buffer.write_variant(name_reference);
        } else {
            buffer.write_variant(0);
        }

        let has_lines = proto.line_info.len() > 0 || proto.absolute_line_info.len() > 0;
        if has_lines {
            buffer.write::<u8>(1);
            buffer.write::<u8>(proto.linegaplog2);

            for i in 0..proto.instructions.len() {
                buffer.write::<u8>(proto.line_info[i]);
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
                let name_reference = self.string_reference(&local.name);
                buffer.write_variant(name_reference);
                buffer.write_variant(local.start_pc);
                buffer.write_variant(local.end_pc);
                buffer.write(local.register);
            }

            buffer.write_variant(proto.upvalues.len() as u32);
            for upvalue in proto.upvalues.iter() {
                let reference = self.string_reference(upvalue);
                buffer.write_variant(reference);
            }
        } else {
            buffer.write::<u8>(0);
        }
    }

    fn string_from_reference(&self, buffer: &mut Buffer) -> Option<String> {
        let id = buffer.read_variant();
        if id == 0 { return None }

        Some(self.strings.get(id as usize -1).unwrap().to_string())
    }

    fn string_reference(&self, string: &str) -> u32 {
        self.strings.iter().position(|s| s == string).unwrap() as u32 + 1
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

    fn read_string(&mut self) -> String;
    fn write_string(&mut self, string: &str);
}

impl Variant for Buffer {
    fn read_variant(&mut self) -> u32 {
        let mut value = 0;
        let mut shift = 0;

        loop {
            let byte = self.read::<u8>();
            value |= ((byte & 127) as u32) << shift;
            shift += 7;

            if (byte & 128) == 0 { break }
        }

        value
    }

    fn write_variant(&mut self, mut value: u32) {
        loop {
            self.write::<u8>(((value & 127) | (((value > 127) as u32) << 7)) as u8);
            value >>= 7;
            if value == 0 { break }
        }
    }

    fn read_string(&mut self) -> String {
        let mut bytes = Vec::new();

        let length = self.read_variant();
        for _ in 0..length {
            bytes.push(self.read::<u8>());
        }

        String::from_utf8(bytes).unwrap()
    }

    fn write_string(&mut self, string: &str) {
        let bytes = string.as_bytes();
        self.write_variant(bytes.len() as u32);
        for byte in bytes {
            self.write(*byte);
        }
    }
}
