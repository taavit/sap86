use crate::{
    emulator::{
        cpu::Cpu,
        machine::{CursorPosition, Machine},
        storage::Floppy,
    },
    isa::registers::{Register8, Register16, SegmentRegister},
};

pub struct Bios;

impl Bios {
    pub fn handle_interrupt(int: u8, cpu: &mut Cpu, machine: &mut Machine) {
        match int {
            0x10 => {
                let op = cpu.registers.read_u8(Register8::Ah);
                match op {
                    0x00 => {
                        let video_mode = cpu.registers.read_u8(Register8::Al);
                        machine.video.set_video_mode(video_mode);
                        println!("Set video mode to: {}", video_mode);
                    }
                    0x02 => {
                        let cursor_position = CursorPosition {
                            page: cpu.registers.read_u8(Register8::Bh),
                            row: cpu.registers.read_u8(Register8::Dh),
                            col: cpu.registers.read_u8(Register8::Dl),
                        };
                        machine.video.set_cursor_position(cursor_position);
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
                        let drive = cpu.registers.read_u8(Register8::Dl);
                        let head = cpu.registers.read_u8(Register8::Dh);
                        let offset = cpu.registers.read_u16(Register16::Bx);
                        let segment = cpu.registers.read_segment(SegmentRegister::Es);

                        println!(
                            "Reading {count} sector(s) from {cylinder}:{head}:{sector} into {segment:04X}:{offset:04X} from {drive:02X}"
                        );
                        let bytes = machine
                            .floppy
                            .read_chs_sector(cylinder, head, sector)
                            .to_vec();
                        for byte in bytes {
                            machine.write_physical_u8((segment as u32 * 16) + offset as u32, byte);
                        }
                        cpu.flags.carry = false;
                    }
                    _ => panic!("Unhandled interrupt {:02X}:{:02X}", int, op),
                }
            }

            _ => panic!("Unhandled interrupt {:02X} group", int),
        }
    }
}
