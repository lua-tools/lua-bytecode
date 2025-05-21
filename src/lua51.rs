use crate::*;
use buffer::Buffer;

pub trait LuaBytecode {
    fn from(data: &[u8]) -> Result<Bytecode, String>;
    fn parse_header(&self, buffer: &mut Buffer) -> Header;
    fn parse_proto(&mut self, buffer: &mut Buffer) -> Proto;

    fn write(&mut self) -> Vec<u8>;
    fn write_proto(&self, index: u32, buffer: &mut Buffer);
}

impl LuaBytecode for Bytecode {
    fn from(data: &[u8]) -> Result<Bytecode, String> {
        let mut bytecode = Bytecode::default();
        let mut buffer = Buffer::new(data.to_vec());

        bytecode.header = bytecode.parse_header(&mut buffer);
        let main_proto = bytecode.parse_proto(&mut buffer);
        bytecode.protos.push(main_proto);

        Ok(bytecode)
    }

    fn parse_header(&self, buffer: &mut Buffer) -> Header {
        let magic = buffer.read::<u32>();
        assert_eq!(magic, LUA_MAGIC);

        let version = buffer.read::<u8>();
        assert_eq!(version, 0x51);

        Header {
            version,
            format: buffer.read::<u8>(),
            is_big_endian: buffer.read::<bool>(),
            int_size: buffer.read::<u8>(),
            size_t_size: buffer.read::<u8>(),
            instruction_size: buffer.read::<u8>(),
            number_size: buffer.read::<u8>(),
            is_number_integral: buffer.read::<bool>(),
            luajit_flags: 0,
        }
    }

    fn parse_proto(&mut self, buffer: &mut Buffer) -> Proto {
        let mut proto = Proto::default();

        proto.name = Some(buffer.read_string());
        proto.line_defined = buffer.read::<u32>();
        proto.last_line_defined = buffer.read::<u32>();
        proto.upvalue_count = buffer.read::<u8>();
        proto.parameter_count = buffer.read::<u8>();
        proto.is_vararg = buffer.read::<bool>();
        proto.max_stack_size = buffer.read::<u8>();

        let instruction_count = buffer.read::<u32>();
        for _ in 0..instruction_count {
            let instruction = buffer.read::<u32>();
            let instruction = Instruction::from_bytes(&instruction.to_le_bytes());
            proto.instructions.push(instruction);
        }

        let constant_count = buffer.read::<u32>();
        for _ in 0..constant_count {
            let kind = buffer.read::<u8>();

            let constant = match kind {
                constant::LUA_CONSTANT_NIL => Constant::Nil,
                constant::LUA_CONSTANT_BOOLEAN => Constant::Bool(buffer.read::<u8>() > 0),
                constant::LUA_CONSTANT_NUMBER => Constant::Number(buffer.read::<f64>()),
                constant::LUA_CONSTANT_STRING => Constant::String(buffer.read_string()),

                _ => {
                    unreachable!();
                }
            };

            proto.constants.push(constant);
        }

        let proto_count = buffer.read::<u32>();
        for _ in 0..proto_count {
            proto.protos.push(self.protos.len() as u32); // proto_id
            let child_proto = self.parse_proto(buffer);
            self.protos.push(child_proto);
        }

        let line_info_count = buffer.read::<u32>();
        for _ in 0..line_info_count {
            proto.line_info.push(buffer.read::<u32>());
        }

        let local_count = buffer.read::<u32>();
        for _ in 0..local_count {
            proto.locals.push(LocalVariable {
                name: buffer.read_string(),
                start_pc: buffer.read::<u32>(),
                end_pc: buffer.read::<u32>(),
                #[cfg(feature = "luau")]
                register: 0,
            })
        }

        let upvalue_count = buffer.read::<u32>();
        for _ in 0..upvalue_count {
            proto.upvalues.push(buffer.read_string());
        }

        proto
    }

    fn write(&mut self) -> Vec<u8> {
        let mut buffer = Buffer::new(Vec::new());

        buffer.write::<u32>(LUA_MAGIC);
        buffer.write::<u8>(self.header.version);
        buffer.write::<u8>(self.header.format);
        buffer.write::<bool>(self.header.is_big_endian);
        buffer.write::<u8>(self.header.int_size);
        buffer.write::<u8>(self.header.size_t_size);
        buffer.write::<u8>(self.header.instruction_size);
        buffer.write::<u8>(self.header.number_size);
        buffer.write::<bool>(self.header.is_number_integral);

        self.write_proto(self.main_proto_id, &mut buffer);

        buffer.set_position(0);
        buffer.read_all()
    }

    fn write_proto(&self, index: u32, buffer: &mut Buffer) {
        let proto = &self.protos[index as usize];

        buffer.write_string(proto.name.clone().unwrap());
        buffer.write::<u32>(proto.line_defined);
        buffer.write::<u32>(proto.last_line_defined);
        buffer.write::<u8>(proto.upvalue_count);
        buffer.write::<u8>(proto.parameter_count);
        buffer.write::<bool>(proto.is_vararg);
        buffer.write::<u8>(proto.max_stack_size);

        buffer.write::<u32>(proto.instructions.len() as u32);
        for instruction in proto.instructions.iter() {
            buffer.write::<i32>(instruction.0 as i32);
        }

        buffer.write::<u32>(proto.constants.len() as u32);
        for constant in proto.constants.iter() {
            buffer.write::<u8>(constant.kind());

            match constant {
                Constant::Nil => (),
                Constant::Bool(value) => {
                    buffer.write(*value);
                }

                Constant::Number(value) => {
                    buffer.write(*value);
                }

                Constant::String(value) => {
                    buffer.write_string(value.clone());
                }

                _ => unreachable!(),
            }
        }

        buffer.write::<u32>(proto.protos.len() as u32);
        for proto in proto.protos.iter() {
            self.write_proto(*proto, buffer);
        }

        buffer.write::<u32>(proto.line_info.len() as u32);
        for line in proto.line_info.iter() {
            buffer.write::<u32>(*line);
        }

        buffer.write::<u32>(proto.locals.len() as u32);
        for local in proto.locals.iter() {
            buffer.write_string(local.name.clone());
            buffer.write::<u32>(local.start_pc);
            buffer.write::<u32>(local.end_pc);
        }

        buffer.write::<u32>(proto.upvalues.len() as u32);
        for upvalue in proto.upvalues.iter() {
            buffer.write_string(upvalue.clone());
        }
    }
}

trait LuaString {
    fn read_string(&mut self) -> RawLuaString;
    fn write_string(&mut self, string: RawLuaString);
}

impl LuaString for Buffer {
    fn read_string(&mut self) -> RawLuaString {
        let mut bytes = Vec::new();

        let length = self.read::<u64>();
        for _ in 0..length {
            bytes.push(self.read::<u8>());
        }

        bytes
    }

    fn write_string(&mut self, string: RawLuaString) {
        self.write::<u64>(string.len() as u64);
        for byte in string {
            self.write::<u8>(byte);
        }
    }
}
