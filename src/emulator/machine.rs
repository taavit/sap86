use core::panic;

use crate::emulator::{cpu::Cpu, memory::Memory};

pub struct Machine {
    pub memory: Memory,
}

impl Machine {
    pub fn boot_program(&mut self, cpu: &mut Cpu, program: &[u8]) {
        self.memory.load_program(program);
        cpu.registers.set_ip(0x7C00);
    }

    pub fn read_physical_u8(&self, addr: u32) -> u8 {
        match addr {
            0x00000..=0x003FF => panic!("Access to IVT (Interrupt Vector Table)"),
            0x00400..=0x004FF => panic!("Access to BDA (BIOS Data area)"),
            0x00500..=0x9FFFF => self.memory.read_u8(addr),
            0xA0000..=0xB7FFF => panic!("Access to Video Ram Graphic"),
            0xB8000..=0xBFFFF => panic!("Access to Video Ram Text"),
            0xC0000..=0xEFFFF => panic!("Access to Option ROM"),
            0xF0000..=0xFFFEF => panic!("Access to System BIOS ROM"),
            0xFFFF0..=0xFFFFF => panic!("Access to Reset Vector"),
            _ => panic!("Access outside of available addresses"),
        }
    }

    pub fn read_physical_u16(&self, addr: u32) -> u16 {
        match addr {
            0x00000..=0x003FF => panic!("Access to IVT (Interrupt Vector Table)"),
            0x00400..=0x004FF => panic!("Access to BDA (BIOS Data area)"),
            0x00500..=0x9FFFF => self.memory.read_u16(addr),
            0xA0000..=0xB7FFF => panic!("Access to Video Ram Graphic"),
            0xB8000..=0xBFFFF => panic!("Access to Video Ram Text"),
            0xC0000..=0xEFFFF => panic!("Access to Option ROM"),
            0xF0000..=0xFFFEF => panic!("Access to System BIOS ROM"),
            0xFFFF0..=0xFFFFF => panic!("Access to Reset Vector"),
            _ => panic!("Access outside of available addresses"),
        }
    }

    pub fn write_physical_u8(&mut self, addr: u32, value: u8) {
        match addr {
            0x00000..=0x003FF => panic!("Access to IVT (Interrupt Vector Table)"),
            0x00400..=0x004FF => panic!("Access to BDA (BIOS Data area)"),
            0x00500..=0x9FFFF => self.memory.write_u8(addr, value),
            0xA0000..=0xB7FFF => panic!("Access to Video Ram Graphic"),
            0xB8000..=0xBFFFF => panic!("Access to Video Ram Text"),
            0xC0000..=0xEFFFF => panic!("Access to Option ROM"),
            0xF0000..=0xFFFEF => panic!("Access to System BIOS ROM"),
            0xFFFF0..=0xFFFFF => panic!("Access to Reset Vector"),
            _ => panic!("Access outside of available addresses"),
        }
    }

    pub fn write_physical_u16(&mut self, addr: u32, value: u16) {
        match addr {
            0x00000..=0x003FF => panic!("Access to IVT (Interrupt Vector Table)"),
            0x00400..=0x004FF => panic!("Access to BDA (BIOS Data area)"),
            0x00500..=0x9FFFF => self.memory.write_u16(addr, value),
            0xA0000..=0xB7FFF => panic!("Access to Video Ram Graphic"),
            0xB8000..=0xBFFFF => panic!("Access to Video Ram Text"),
            0xC0000..=0xEFFFF => panic!("Access to Option ROM"),
            0xF0000..=0xFFFEF => panic!("Access to System BIOS ROM"),
            0xFFFF0..=0xFFFFF => panic!("Access to Reset Vector"),
            _ => panic!("Access outside of available addresses"),
        }
    }
}
