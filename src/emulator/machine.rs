use core::panic;

use crate::{
    emulator::{bios::Bios, cpu::Cpu, memory::Memory, storage::Floppy525DD},
    isa::registers::Register16,
};

pub struct Machine {
    pub memory: Memory,
    pub floppy: Floppy525DD,
    pub video: VideoCard,
}

#[derive(Debug, Default)]
pub struct CursorPosition {
    pub page: u8,
    pub row: u8,
    pub col: u8,
}

#[derive(Debug, Default)]
pub struct VideoCard {
    video_mode: u8,
    cursor_position: CursorPosition,
}

impl VideoCard {
    pub fn set_video_mode(&mut self, video_mode: u8) {
        self.video_mode = video_mode;
    }

    pub fn set_cursor_position(&mut self, cursor_position: CursorPosition) {
        self.cursor_position = cursor_position;
    }
}

impl Machine {
    pub fn boot(&mut self, cpu: &mut Cpu, device: &[u8]) {
        self.floppy.insert(device);
        self.memory.load_program(&device[..512]);
        cpu.registers.set_ip(0x7C00);
        cpu.registers.write_u16(Register16::Sp, 0xFFFE);
    }

    pub fn handle_bios_interrupt(&mut self, cpu: &mut Cpu, int: u8) {
        Bios::handle_interrupt(int, cpu, self);
    }

    pub fn read_physical_u8(&self, addr: u32) -> u8 {
        match addr {
            0x00000..=0x003FF => self.memory.read_u8(addr),
            0x00400..=0x004FF => panic!("{addr:02X}: Access to BDA (BIOS Data area)"),
            0x00500..=0x9FFFF => self.memory.read_u8(addr),
            0xA0000..=0xB7FFF => panic!("{addr:02X}: Access to Video Ram Graphic"),
            0xB8000..=0xBFFFF => panic!("{addr:02X}: Access to Video Ram Text"),
            0xC0000..=0xEFFFF => panic!("{addr:02X}: Access to Option ROM"),
            0xF0000..=0xFFFEF => panic!("{addr:02X}: Access to System BIOS ROM"),
            0xFFFF0..=0xFFFFF => panic!("{addr:02X}: Access to Reset Vector"),
            _ => panic!("{addr:02X}: Access outside of available addresses"),
        }
    }

    pub fn read_physical_u16(&self, addr: u32) -> u16 {
        match addr {
            0x00000..=0x003FF => self.memory.read_u16(addr),
            0x00400..=0x004FF => panic!("{addr:02X}: Access to BDA (BIOS Data area)"),
            0x00500..=0x9FFFF => self.memory.read_u16(addr),
            0xA0000..=0xB7FFF => panic!("{addr:02X}: Access to Video Ram Graphic"),
            0xB8000..=0xBFFFF => panic!("{addr:02X}: Access to Video Ram Text"),
            0xC0000..=0xEFFFF => panic!("{addr:02X}: Access to Option ROM"),
            0xF0000..=0xFFFEF => panic!("{addr:02X}: Access to System BIOS ROM"),
            0xFFFF0..=0xFFFFF => panic!("{addr:02X}: Access to Reset Vector"),
            _ => panic!("Access outside of available addresses"),
        }
    }

    pub fn write_physical_u8(&mut self, addr: u32, value: u8) {
        match addr {
            0x00000..=0x003FF => self.memory.write_u8(addr, value),
            0x00400..=0x004FF => panic!("{addr:02X}: Write access to BDA (BIOS Data area)"),
            0x00500..=0x9FFFF => self.memory.write_u8(addr, value),
            0xA0000..=0xB7FFF => panic!("{addr:02X}: Write access to Video Ram Graphic"),
            0xB8000..=0xBFFFF => panic!("{addr:02X}: Write access to Video Ram Text"),
            0xC0000..=0xEFFFF => panic!("{addr:02X}: Write access to Option ROM"),
            0xF0000..=0xFFFEF => panic!("{addr:02X}: Write access to System BIOS ROM"),
            0xFFFF0..=0xFFFFF => panic!("{addr:02X}: Write access to Reset Vector"),
            _ => panic!("Access outside of available addresses"),
        }
    }

    pub fn write_physical_u16(&mut self, addr: u32, value: u16) {
        match addr {
            0x00000..=0x003FF => self.memory.write_u16(addr, value),
            0x00400..=0x004FF => panic!("{addr:02X}: Write access to BDA (BIOS Data area)"),
            0x00500..=0x9FFFF => self.memory.write_u16(addr, value),
            0xA0000..=0xB7FFF => panic!("{addr:02X}: Write access to Video Ram Graphic"),
            0xB8000..=0xBFFFF => panic!("{addr:02X}: Write access to Video Ram Text"),
            0xC0000..=0xEFFFF => panic!("{addr:02X}: Write access to Option ROM"),
            0xF0000..=0xFFFEF => panic!("{addr:02X}: Write access to System BIOS ROM"),
            0xFFFF0..=0xFFFFF => panic!("{addr:02X}: Write access to Reset Vector"),
            _ => panic!("Access outside of available addresses"),
        }
    }
}
