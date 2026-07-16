use std::fs::read_to_string;

use crate::{compiler::compile_program, registers::{Register16, Registers}};

mod compiler;
mod registers;

struct Flags {
    zero: bool,
}

struct Cpu {
    flags: Flags,
    registers: Registers,
    halted: bool,
}

#[derive(Debug)]
enum Op {
    Nop,
    Dec { dst: Register16 },
    Inc { dst: Register16 },
    Ldi { imm: u8 },
    Lea { addr: u8 },
    Jnz { addr: u8 },
    Jz { addr: u8 },
    Jmp { addr: u8 },
    Ld { src: Register16 },
    Test { src: Register16 },
    Mov { src: Register16, dst: Register16 },
    Int(u8),
    Halt,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            flags: Flags { zero: false },
            registers: Registers::new(),
            halted: false,
        }
    }

    pub fn execute(&mut self, machine: &mut Machine, instruction: Op) {
        match instruction {
            Op::Halt => {
                self.halted = true;
            }
            Op::Lea { addr } => {
                self.registers.write_u16(Register16::Ax, addr as u16);
            }
            Op::Ldi { imm } => {
                self.registers.write_u16(Register16::Ax, imm as u16);
            }
            Op::Dec { dst } => {
                let dst_val = self.registers.read_u16(dst);
                let new_val = dst_val.wrapping_sub(1);
                self.registers.write_u16(dst, new_val);
                self.flags.zero = new_val == 0;
            }
            Op::Inc { dst } => {
                let dst_val = self.registers.read_u16(dst);
                let new_val = dst_val.wrapping_add(1);
                self.registers.write_u16(dst, new_val);
                self.flags.zero = new_val == 0;
            }
            Op::Jnz { addr } => {
                if !self.flags.zero {
                    self.registers.set_ip(addr as u16);
                }
            }
            Op::Jz { addr } => {
                if self.flags.zero {
                    self.registers.set_ip(addr as u16);
                }
            }
            Op::Jmp { addr } => {
                self.registers.set_ip(addr as u16);
            }
            Op::Test { src } => {
                self.flags.zero = self.registers.read_u16(src) == 0;
            }
            Op::Ld { src } => {
                let addr = self.registers.read_u16(src);
                let value = machine.memory.read_u8(addr);
                self.registers.write_u16(Register16::Ax, value as u16);
            }
            Op::Mov { dst, src } => {
                let v = self.registers.read_u16(src);
                self.registers.write_u16(dst, v);
            }
            Op::Int(int) => match int {
                0x10 => {
                    print!("{}", self.registers.read_u16(Register16::Ax) as u8 as char);
                }
                _ => {
                    panic!("Invalid interrupt");
                }
            },
            Op::Nop => {}
        }
    }

    pub fn fetch_decode(&mut self, machine: &mut Machine) -> Op {
        let v = machine.read_u8(self);
        match v {
            0x00 => Op::Nop,
            0x10 => Op::Ldi {
                imm: machine.read_u8(self),
            },
            0x11 => Op::Lea {
                addr: machine.read_u8(self),
            },
            0x20..=0x2F => {
                let src = match (v & 0x0C) >> 2 {
                    0 => Register16::Ax,
                    1 => Register16::Cx,
                    2 => Register16::Dx,
                    3 => Register16::Bx,
                    _ => unreachable!(),
                };
                let dst = match v & 0x03 {
                    0 => Register16::Ax,
                    1 => Register16::Cx,
                    2 => Register16::Dx,
                    3 => Register16::Bx,
                    _ => unreachable!(),
                };
                Op::Mov { src, dst }
            }
            0xCD => Op::Int(machine.read_u8(self)),
            0xCC => Op::Int(0x03),
            0x40..=0x47 => {
                let dst = Register16::from(v & 0x07);
                Op::Inc { dst }
            }
            0x48..=0x4F => {
                let dst = Register16::from(v & 0x07);
                Op::Dec { dst }
            }
            0x50 => Op::Jnz {
                addr: machine.read_u8(self),
            },
            0x51 => Op::Jz {
                addr: machine.read_u8(self),
            },
            0x52 => Op::Jmp {
                addr: machine.read_u8(self),
            },
            0x60..=0x63 => {
                let src = match v & 0x03 {
                    0 => Register16::Ax,
                    1 => Register16::Cx,
                    2 => Register16::Dx,
                    3 => Register16::Bx,
                    _ => unreachable!(),
                };
                Op::Ld { src }
            }
            0x70..=0x73 => {
                let src = match v & 0x03 {
                    0 => Register16::Ax,
                    1 => Register16::Cx,
                    2 => Register16::Dx,
                    3 => Register16::Bx,
                    _ => unreachable!(),
                };
                Op::Test { src }
            }
            0xF4 => Op::Halt,
            i => panic!("Unkown command: {i}"),
        }
    }
}

#[derive(Debug)]
struct Memory {
    memory: [u8; u16::MAX as usize],
}

impl Memory {
    pub fn new() -> Self {
        Self { memory: [0; u16::MAX as usize] }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn load_program(&mut self, program: &[u8]) {
        if program.len() > self.memory.len() {
            panic!("Program to big");
        }
        self.memory[..program.len()].copy_from_slice(program);
    }
}

struct Machine {
    memory: Memory,
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
}

fn main() {
    let mut cpu = Cpu::new();
    let memory = Memory::new();
    let mut machine = Machine { memory };
    let path: Vec<String> = std::env::args().collect();
    let program = read_to_string(&path[1]).unwrap();
    let compiled = compile_program(&program);
    eprintln!("[COMPILER] Size = {}", compiled.len());
    machine.load_program(&compiled);
    loop {
        let instruction = cpu.fetch_decode(&mut machine);

        cpu.execute(&mut machine, instruction);
        if cpu.halted {
            break;
        }
    }
}
