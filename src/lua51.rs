use crate::*;
use buffer::Buffer;


pub trait LuaBytecode {
    fn new() -> Self;
    fn parse(&mut self, data: &[u8]);
    fn parse_header(&self, buffer: &mut Buffer) -> Header;
    fn parse_proto(&mut self, buffer: &mut Buffer) -> Proto;
}

impl LuaBytecode for Bytecode {
    fn new() -> Self {
        Default::default()
    }

    fn parse(&mut self, data: &[u8]) {
        let mut buffer = Buffer::new(data.to_vec());

        self.header = self.parse_header(&mut buffer);
        let main_proto = self.parse_proto(&mut buffer);
        self.protos.push(main_proto);
    }

    fn parse_header(&self, buffer: &mut Buffer) -> Header {
        let magic = buffer.read::<u32>();
        assert_eq!(magic, 0x61754c1b);

        let version = buffer.read::<u8>();
        assert_eq!(version, 0x51);

        let format = buffer.read::<u8>();
        let is_big_endian = buffer.read::<bool>();
        let int_size = buffer.read::<u8>();
        let size_t_size = buffer.read::<u8>();
        let instruction_size = buffer.read::<u8>();
        let number_size = buffer.read::<u8>();
        let is_number_integral = buffer.read::<bool>();

        Header {
            version,
            format,
            is_big_endian,
            is_number_integral,
            int_size,
            size_t_size,
            instruction_size,
            number_size,
            luajit_flags: 0
        }
    }

    fn parse_proto(&mut self, buffer: &mut Buffer) -> Proto {
        let mut proto = Proto::new();

        proto.name = Some(buffer.read_string());
        proto.line_defined = buffer.read::<u32>();
        proto.last_line_defined = buffer.read::<u32>();
        let upvalues_count = buffer.read::<u8>();
        proto.parameters_count = buffer.read::<u8>();
        proto.is_vararg = buffer.read::<bool>();
        proto.max_stack_size = buffer.read::<u8>();

        let instructions_count = buffer.read::<u32>();
        for _ in 0..instructions_count {
            let instruction = buffer.read::<u32>();
            let instruction = Instruction::from_bytes(&instruction.to_le_bytes());
            proto.instructions.push(instruction);
        }

        let constants_count = buffer.read::<u32>();
        for _ in 0..constants_count {
            let mut constant = Constant::new();
            constant.kind = buffer.read::<u8>();

            match constant.kind {
                LUA_CONSTANT_NIL => (),

                LUA_CONSTANT_BOOLEAN => {
                    constant.value = buffer.read::<u8>().to_le_bytes().to_vec();
                }

                LUA_CONSTANT_NUMBER => {
                    constant.value = buffer.read::<f64>().to_le_bytes().to_vec();
                }

                LUA_CONSTANT_STRING => {
                    constant.value = buffer.read_string().as_bytes().to_vec();
                }

                _ => {
                    unreachable!();
                }
            }
        }

        let protos_count = buffer.read::<u32>();
        for _ in 0..protos_count {
            proto.protos.push(self.protos.len() as u32); // proto_id
            let child_proto = self.parse_proto(buffer);
            self.protos.push(child_proto);
        }

        let line_info_count = buffer.read::<u32>();
        for _ in 0..line_info_count {
            proto.line_info.push(buffer.read::<u8>());
        }

        let locals_count = buffer.read::<u32>();
        for _ in 0..locals_count {
            proto.locals.push(LocalVariable {
                name: buffer.read_string(),
                start_pc: buffer.read::<u32>(),
                end_pc: buffer.read::<u32>(),
                #[cfg(feature = "luau")]
                register: 0
            })
        }

        let upvalues_count = buffer.read::<u32>();
        for _ in 0..upvalues_count {
            proto.upvalues.push(buffer.read_string());
        }

        proto
    }
}

trait LuaString {
    fn read_string(&mut self) -> String;
}

impl LuaString for Buffer {
    fn read_string(&mut self) -> String {
        let mut bytes = Vec::new();

        let length = self.read::<u64>();
        for _ in 0..length {
            bytes.push(self.read::<u8>());
        }

        String::from_utf8(bytes).unwrap()
    }
}
