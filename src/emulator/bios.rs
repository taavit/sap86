use std::time::SystemTime;

use crate::{
    emulator::{
        cpu::Cpu,
        machine::{CursorPosition, Machine},
    },
    isa::registers::{Register8, Register16, SegmentRegister},
};

pub struct Bios;

impl Bios {
    pub fn init_memory(machine: &mut Machine) {
        let mut offset = 0xE000;
        machine.write_physical_u8(Cpu::calculate_physical_address(0xF000, offset), 8);
        machine.write_physical_u8(Cpu::calculate_physical_address(0xF000, offset + 1), 0xFC);
        machine.write_physical_u8(Cpu::calculate_physical_address(0xF000, offset + 2), 0);
        machine.write_physical_u8(Cpu::calculate_physical_address(0xF000, offset + 3), 1);
        machine.write_physical_u8(Cpu::calculate_physical_address(0xF000, offset + 4), 0);
        machine.write_physical_u8(Cpu::calculate_physical_address(0xF000, offset + 5), 0);
        machine.write_physical_u8(Cpu::calculate_physical_address(0xF000, offset + 6), 0);
        machine.write_physical_u8(Cpu::calculate_physical_address(0xF000, offset + 7), 0);
    }
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
                    0x03 => {
                        let pos = machine.video.get_cursor_position();
                        cpu.registers.write_u8(Register8::Dh, pos.row);
                        cpu.registers.write_u8(Register8::Dl, pos.col);

                        cpu.registers.write_u8(Register8::Ch, 6);
                        cpu.registers.write_u8(Register8::Cl, 7);
                    }
                    0x09 => {
                        let character = cpu.registers.read_u8(Register8::Al) as char;
                        let count = cpu.registers.read_u16(Register16::Cx);
                        let attribute = cpu.registers.read_u8(Register8::Bl);

                        // Parse the VGA attribute byte
                        let fg = attribute & 0x0F;
                        let bg = (attribute & 0x70) >> 4;
                        let blink = (attribute & 0x80) != 0;

                        // Map standard CGA/VGA colors to ANSI foreground codes
                        let ansi_fg = match fg {
                            0 => 30,  // Black
                            1 => 34,  // Blue
                            2 => 32,  // Green
                            3 => 36,  // Cyan
                            4 => 31,  // Red
                            5 => 35,  // Magenta
                            6 => 33,  // Brown
                            7 => 37,  // Light Gray
                            8 => 90,  // Dark Gray (Bright Black)
                            9 => 94,  // Bright Blue
                            10 => 92, // Bright Green
                            11 => 96, // Bright Cyan
                            12 => 91, // Bright Red
                            13 => 95, // Bright Magenta
                            14 => 93, // Yellow (Bright Brown)
                            15 => 97, // White (Bright Light Gray)
                            _ => 37,
                        };

                        // Map standard CGA/VGA colors to ANSI background codes
                        let ansi_bg = match bg {
                            0 => 40, // Black
                            1 => 44, // Blue
                            2 => 42, // Green
                            3 => 46, // Cyan
                            4 => 41, // Red
                            5 => 45, // Magenta
                            6 => 43, // Brown
                            7 => 47, // Light Gray
                            _ => 40,
                        };

                        // Add ANSI blink modifier if the highest bit was set
                        let blink_code = if blink { ";5" } else { "" };

                        // Construct the escape sequence
                        let color_prefix = format!("\x1b[{};{}{}m", ansi_fg, ansi_bg, blink_code);
                        let reset_suffix = "\x1b[0m";

                        for _ in 0..count {
                            print!("{}{}{}", color_prefix, character, reset_suffix);
                        }
                    }
                    0x0E => print!("{}", cpu.registers.read_u8(Register8::Al) as char),
                    0x0F => {
                        cpu.registers.write_u8(Register8::Al, 0x03);
                        cpu.registers.write_u8(Register8::Ah, 80);
                        cpu.registers.write_u8(Register8::Bh, 0);
                    }
                    _ => panic!("Unhandled interrupt {:02X}:{:02X}", int, op),
                }
            }
            0x11 => {
                // INT 11h - BIOS Equipment Determination
                // Returns equipment list in the AX register:
                // Bit 0    : 1 = Floppy drive(s) installed
                // Bit 1    : 0 = Math coprocessor not installed
                // Bits 2-3 : System RAM banks (legacy)
                // Bits 4-5 : Initial video mode (10 = 80x25 Color)
                // Bits 6-7 : Number of floppy drives minus 1 (00 = 1 drive)
                // Bit 8    : 0 = DMA present
                // Bits 9-11: Number of serial ports (000 = 0 ports)
                // Bit 12   : 0 = No game port
                // Bit 13   : 0 = No internal modem
                // Bits14-15: Number of parallel ports (00 = 0 ports)

                // 0x0021 corresponds to: 1 Floppy Drive, 80x25 Color Video
                cpu.registers.write_u16(Register16::Ax, 0x0021);
            }
            0x12 => {
                cpu.registers.write_u16(Register16::Ax, 0x0280);
            }
            0x13 => {
                let op = cpu.registers.read_u8(Register8::Ah);
                match op {
                    0x00 => {
                        println!("[EMU ] Reset drive");
                        cpu.flags.carry = false;
                        cpu.registers.write_u8(Register8::Ah, 0);
                    }
                    0x02 => {
                        let count = cpu.registers.read_u8(Register8::Al);
                        let ch = cpu.registers.read_u8(Register8::Ch);
                        let cl = cpu.registers.read_u8(Register8::Cl);
                        let drive = cpu.registers.read_u8(Register8::Dl);
                        let head = cpu.registers.read_u8(Register8::Dh);
                        let mut offset = cpu.registers.read_u16(Register16::Bx);
                        let segment = cpu.registers.read_segment(SegmentRegister::Es);
                        let Some(floppy) = machine.floppy.as_ref() else {
                            eprintln!("[EMU ] Floppy not inserted");
                            cpu.registers.write_u8(Register8::Ah, 0x01);
                            return;
                        };
                        let cylinder = ((cl as u16 & 0xC0) << 2) | ch as u16;
                        let sector = cl & 0x3F;
                        eprintln!(
                            "[EMU ] Reading {count} sector(s) from {cylinder}:{head}:{sector} into {segment:04X}:{offset:04X} from {drive:02X}"
                        );
                        let bytes = floppy
                            .read_chs_sectors(cylinder, head, sector, count)
                            .to_vec();
                        for byte in bytes {
                            machine.write_physical_u8((segment as u32 * 16) + offset as u32, byte);
                            offset += 1;
                        }
                        cpu.flags.carry = false;
                        cpu.registers.write_u8(Register8::Ah, 0);
                    }
                    0x08 => {
                        let drive = cpu.registers.read_u8(Register8::Dl);
                        cpu.registers.write_u8(Register8::Ah, 0);
                        cpu.registers.write_u8(Register8::Al, 0);
                        cpu.flags.carry = false;
                        cpu.registers.write_u8(Register8::Ch, 80);
                        cpu.registers.write_u8(Register8::Cl, 18);
                        cpu.registers.write_u8(Register8::Dh, 2);
                        cpu.registers.write_u8(Register8::Dl, 1);
                        eprintln!("Reading drive param. Drive: {drive:02X}")
                    }
                    0x15 => {
                        let drive = cpu.registers.read_u8(Register8::Dl);
                        cpu.flags.carry = false;
                        match drive {
                            0 => {
                                cpu.registers.write_u8(Register8::Ah, 1);
                            }
                            _ => {
                                cpu.registers.write_u8(Register8::Ah, 0);
                            }
                        }

                        eprintln!("[{int:02X}h:{op:02X}h]Reading drive size. Drive: {drive:02X}");
                    }
                    _ => panic!("Unhandled interrupt {:02X}:{:02X}", int, op),
                }
            }
            0x14 => {
                let op = cpu.registers.read_u8(Register8::Ah);
                match op {
                    0x00 => {
                        cpu.registers.write_u8(Register8::Ah, 0);
                        cpu.registers.write_u8(Register8::Al, 0);
                    }
                    _ => panic!("Unhandled interrupt {:02X}:{:02X}", int, op),
                }
            }
            0x15 => {
                let op = cpu.registers.read_u8(Register8::Ah);
                match op {
                    0xC0 => {
                        cpu.registers.write_segment(SegmentRegister::Es, 0xF000);
                        cpu.registers.write_u16(Register16::Bx, 0xE000);
                        cpu.registers.write_u8(Register8::Ah, 0);
                        cpu.registers.write_u8(Register8::Al, 0);
                        cpu.flags.carry = false;
                    }
                    _ => panic!("Unhandled interrupt {:02X}:{:02X}", int, op),
                }
            }
            0x17 => {
                let op = cpu.registers.read_u8(Register8::Ah);
                match op {
                    0x01 => {
                        // cpu.registers.write_u8(Register8::Ah, 0);
                        // cpu.registers.write_u8(Register8::Al, 0);
                    }
                    _ => panic!("Unhandled interrupt {:02X}:{:02X}", int, op),
                }
            }
            0x1A => {
                let op = cpu.registers.read_u8(Register8::Ah);
                match op {
                    0x00 => {
                        cpu.registers.write_u16(Register16::Cx, 0x0F);
                        cpu.registers.write_u16(Register16::Dx, 0x34);
                        cpu.registers.write_u8(Register8::Al, 0x03);
                    }
                    0x02 => {
                        cpu.registers.write_u8(Register8::Ch, 0xA4);
                        cpu.registers.write_u8(Register8::Cl, 0x24);
                        cpu.registers.write_u8(Register8::Dh, 0x00);
                        cpu.registers.write_u8(Register8::Dl, 0x00);
                    }
                    0x04 => {
                        cpu.registers.write_u8(Register8::Ch, 0x15);
                        cpu.registers.write_u8(Register8::Cl, 0x1A);
                        cpu.registers.write_u8(Register8::Dh, 0x07);
                        cpu.registers.write_u8(Register8::Dl, 0x24);
                        cpu.flags.carry = false;
                    }
                    _ => panic!("Unhandled interrupt {:02X}:{:02X}", int, op),
                }
            }

            _ => panic!("Unhandled interrupt {:02X} group", int),
        }
    }
}
