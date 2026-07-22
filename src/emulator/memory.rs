#[derive(Debug)]
pub struct Memory {
    memory: [u8; 1024 * 1024],
}

impl Memory {
    pub fn new() -> Self {
        Self {
            memory: [0; 1024 * 1024],
        }
    }

    pub fn dump(&self) -> &[u8] {
        &self.memory
    }

    pub fn read_u8(&self, addr: u32) -> u8 {
        self.memory[addr as usize]
    }

    pub fn read_u16(&self, addr: u32) -> u16 {
        u16::from_le_bytes([self.memory[addr as usize], self.memory[addr as usize + 1]])
    }

    pub fn write_u8(&mut self, addr: u32, value: u8) {
        self.memory[addr as usize] = value;
    }
    pub fn write_u16(&mut self, addr: u32, val: u16) {
        let splited = val.to_le_bytes();
        self.memory[addr as usize] = splited[0];
        self.memory[addr as usize + 1] = splited[1];
    }

    pub fn load_program(&mut self, program: &[u8]) {
        if program.len() > self.memory.len() - 0x7C00 {
            panic!("Program to big");
        }
        self.memory[0x7C00..0x7C00 + program.len()].copy_from_slice(program);
    }
}
