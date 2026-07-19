use crate::{emulator::cpu::Cpu, isa::registers::Register8};

pub struct Bios;

impl Bios {
    pub fn handle_interrupt(int: u8, cpu: &Cpu) {
        match int {
            0x10 => {
                let op = cpu.registers.read_u8(Register8::Ah);
                match op {
                    0x0E => print!("{}", cpu.registers.read_u8(Register8::Al) as char),
                    _ => panic!("Unhandled interrupt {:02X}:{:02X}", int, op),
                }
            }
            _ => panic!("Unhandled interrupt {:02X} group", int),
        }
    }
}
