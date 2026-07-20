use crate::{
    emulator::cpu::Cpu,
    isa::registers::{Register8, Register16, SegmentRegister},
};

#[derive(Debug, Default)]
struct CursorPosition {
    page: u8,
    row: u8,
    col: u8,
}

pub struct Bios {
    video_mode: u8,
    cursor_position: CursorPosition,
}

impl Bios {
    pub fn new() -> Self {
        Bios {
            video_mode: 0,
            cursor_position: CursorPosition::default(),
        }
    }
}

impl Bios {
    pub fn handle_interrupt(&mut self, int: u8, cpu: &Cpu) {
        match int {
            0x10 => {
                let op = cpu.registers.read_u8(Register8::Ah);
                match op {
                    0x00 => {
                        self.video_mode = cpu.registers.read_u8(Register8::Al);
                        println!("Set video mode to: {}", self.video_mode);
                    }
                    0x02 => {
                        self.cursor_position = CursorPosition {
                            page: cpu.registers.read_u8(Register8::Bh),
                            row: cpu.registers.read_u8(Register8::Dh),
                            col: cpu.registers.read_u8(Register8::Dl),
                        };
                        dbg!(&self.cursor_position);
                    }
                    0x0E => print!("{}", cpu.registers.read_u8(Register8::Al) as char),
                    _ => panic!("Unhandled interrupt {:02X}:{:02X}", int, op),
                }
            }
            0x13 => {
                let op = cpu.registers.read_u8(Register8::Ah);
                match op {
                    0x00 => println!("Reset drive"),
                    0x02 => {
                        let count = cpu.registers.read_u8(Register8::Al);
                        let cylinder = cpu.registers.read_u8(Register8::Ch);
                        let sector = cpu.registers.read_u8(Register8::Cl);
                        let head = cpu.registers.read_u8(Register8::Dh);
                        let buffer = cpu.registers.read_u16(Register16::Bx);
                        let segment = cpu.registers.read_segment(SegmentRegister::Es);

                        println!(
                            "Reading {count} sector(s) from {cylinder}:{head}:{sector} into {segment:04X}:{buffer:04X}"
                        );
                    }
                    _ => panic!("Unhandled interrupt {:02X}:{:02X}", int, op),
                }
            }

            _ => panic!("Unhandled interrupt {:02X} group", int),
        }
    }
}
