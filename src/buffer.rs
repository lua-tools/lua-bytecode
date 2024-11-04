use std::io::{Read, Write};

pub struct Buffer {
    cursor: std::io::Cursor<Vec<u8>>,
}

impl Buffer {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            cursor: std::io::Cursor::new(data)
        }
    }

    pub fn read<T: Copy>(&mut self) -> T {
        let size = std::mem::size_of::<T>();

        let buffer: Vec<u8> = vec![0; size];
        let mut buffer_slice = buffer.into_boxed_slice();

        self.cursor.read(&mut buffer_slice).unwrap();

        let value: &mut T = unsafe { &mut *(buffer_slice.as_mut_ptr() as *mut T) };
        *value
    }

    pub fn read_all(&mut self) -> Vec<u8> {
        let mut buffer = Vec::new();
        self.cursor.read_to_end(&mut buffer).unwrap();

        buffer
    }

    pub fn write<T: Copy>(&mut self, value: T) {
        let size = std::mem::size_of::<T>();
        let buffer = unsafe {
            std::slice::from_raw_parts_mut(&value as *const T as *mut u8, size)
        };

        self.cursor.write_all(&buffer).unwrap();
    }

    pub fn position(&self) -> u64 {
        self.cursor.position()
   }

    pub fn set_position(&mut self, position: u64) {
        self.cursor.set_position(position)
    }

    pub fn advance(&mut self, amount: u64) {
        let position = self.position();
        self.set_position(position + amount);
    }
}
