use std::fs::read_to_string;

use crate::compiler::compile_program;

mod compiler;

struct Registers {
    gpr: [u8; 4],
    ip: u8,
}

impl Registers {
    pub fn step_ip(&mut self) {
        self.ip = self.ip.wrapping_add(1);
    }

    pub fn ip(&self) -> u8 {
        self.ip
    }

    pub fn set_ip(&mut self, ip: u8) {
        self.ip = ip;
    }

    pub fn write_u8(&mut self, reg: Register, value: u8) {
        self.gpr[reg as usize] = value;
    }
    pub fn read_u8(&self, reg: Register) -> u8 {
        self.gpr[reg as usize]
    }
}
struct Flags {
    zero: bool,
}

struct Cpu {
    flags: Flags,
    registers: Registers,
    halted: bool,
}

#[derive(Debug, Clone, Copy)]
enum Register {
    A,
    B,
    C,
    D,
}

#[derive(Debug)]
enum Op {
    Nop,
    Dec { dst: Register },
    Inc { dst: Register },
    Ldi { imm: u8 },
    Lea { addr: u8 },
    Jnz { addr: u8 },
    Jz { addr: u8 },
    Jmp { addr: u8 },
    Ld { src: Register },
    Test { src: Register },
    Mov { src: Register, dst: Register },
    Int(u8),
    Halt,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            flags: Flags { zero: false },
            registers: Registers { gpr: [0; 4], ip: 0 },
            halted: false,
        }
    }

    pub fn execute(&mut self, machine: &mut Machine, instruction: Op) {
        match instruction {
            Op::Halt => {
                self.halted = true;
            }
            Op::Lea { addr } => {
                self.registers.write_u8(Register::A, addr);
            }
            Op::Ldi { imm } => {
                self.registers.write_u8(Register::A, imm);
            }
            Op::Dec { dst } => {
                let dst_val = self.registers.read_u8(dst);
                let new_val = dst_val.wrapping_sub(1);
                self.registers.write_u8(dst, new_val);
                self.flags.zero = new_val == 0;
            }
            Op::Inc { dst } => {
                let dst_val = self.registers.read_u8(dst);
                let new_val = dst_val.wrapping_add(1);
                self.registers.write_u8(dst, new_val);
                self.flags.zero = new_val == 0;
            }
            Op::Jnz { addr } => {
                if !self.flags.zero {
                    self.registers.set_ip(addr);
                }
            }
            Op::Jz { addr } => {
                if self.flags.zero {
                    self.registers.set_ip(addr);
                }
            }
            Op::Jmp { addr } => {
                self.registers.set_ip(addr);
            }
            Op::Test { src } => {
                self.flags.zero = self.registers.read_u8(src) == 0;
            }
            Op::Ld { src } => {
                let addr = self.registers.read_u8(src);
                let value = machine.memory.read_u8(addr);
                self.registers.write_u8(Register::A, value);
            }
            Op::Mov { dst, src } => {
                let v = self.registers.read_u8(src);
                self.registers.write_u8(dst, v);
            }
            Op::Int(int) => match int {
                1 => {
                    print!("{}", self.registers.read_u8(Register::A) as char);
                }
                _ => {
                    panic!("Invalid interrupt");
                }
            },
            Op::Nop => {}
            _ => {}
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
                    0 => Register::A,
                    1 => Register::B,
                    2 => Register::C,
                    3 => Register::D,
                    _ => unreachable!(),
                };
                let dst = match v & 0x03 {
                    0 => Register::A,
                    1 => Register::B,
                    2 => Register::C,
                    3 => Register::D,
                    _ => unreachable!(),
                };
                Op::Mov { src, dst }
            }
            0x30..=0x3F => Op::Int(v & 0x0F),
            0x40..=0x4F => {
                let op: u8 = (v & 0x0C) >> 2;
                let dst = match v & 0x03 {
                    0 => Register::A,
                    1 => Register::B,
                    2 => Register::C,
                    3 => Register::D,
                    _ => unreachable!(),
                };
                match op {
                    0x00 => Op::Inc { dst },
                    0x01 => Op::Dec { dst },
                    _ => unreachable!(),
                }
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
                    0 => Register::A,
                    1 => Register::B,
                    2 => Register::C,
                    3 => Register::D,
                    _ => unreachable!(),
                };
                Op::Ld { src }
            }
            0x70..=0x73 => {
                let src = match v & 0x03 {
                    0 => Register::A,
                    1 => Register::B,
                    2 => Register::C,
                    3 => Register::D,
                    _ => unreachable!(),
                };
                Op::Test { src }
            }
            0xFF => Op::Halt,
            i => panic!("Unkown command: {i}"),
        }
    }
}

#[derive(Debug)]
struct Memory {
    memory: [u8; 256],
}

impl Memory {
    pub fn new() -> Self {
        Self { memory: [0; 256] }
    }

    pub fn read_u8(&self, addr: u8) -> u8 {
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
        let r = self.memory.read_u8(cpu.registers.ip);
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
