use crate::emulator::{cpu::Cpu, memory::Memory};

pub struct Machine {
    pub memory: Memory,
}

impl Machine {
    pub fn load_program(&mut self, program: &[u8]) {
        self.memory.load_program(program);
    }

    pub fn read_u8(&mut self, cpu: &mut Cpu) -> u8 {
        let r = self.memory.read_u8(cpu.registers.ip());
        cpu.registers.step_ip();
        r
    }

    pub fn read_u16(&mut self, cpu: &mut Cpu) -> u16 {
        let l = self.read_u8(cpu);
        let h = self.read_u8(cpu);

        u16::from_le_bytes([l, h])
    }

    pub fn read_rel8(&mut self, cpu: &mut Cpu) -> i16 {
        i8::from_ne_bytes([self.read_u8(cpu)]) as i16
    }
}
