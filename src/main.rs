use std::fs::read_to_string;

use crate::{
    compiler::{ModRm, Operand, compile_program},
    registers::{Register8, Register16, Registers},
};

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
    Lea { addr: u16 },
    Jnz { addr: u16 },
    Jz { addr: u16 },
    Jmp { addr: u16 },
    Ld { src: Register16 },
    Test { op1: Operand, op2: Operand },
    Mov { src: Operand, dst: Operand },
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
                self.registers.write_u16(Register16::Ax, addr);
            }
            Op::Ldi { imm } => {
                self.registers.write_u8(Register8::Al, imm);
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
            Op::Test { op1, op2 } => match (op1, op2) {
                (Operand::Register16(reg1), Operand::Register16(reg2)) => {
                    let val1 = self.registers.read_u16(reg1);
                    let val2 = self.registers.read_u16(reg2);
                    self.flags.zero = (val1 & val2) == 0;
                }
                (Operand::Register8(reg1), Operand::Register8(reg2)) => {
                    let val1 = self.registers.read_u8(reg1);
                    let val2 = self.registers.read_u8(reg2);
                    self.flags.zero = (val1 & val2) == 0;
                }
                _ => panic!("Invalid combination"),
            },
            Op::Ld { src } => {
                let addr = self.registers.read_u16(src);
                let value = machine.memory.read_u8(addr);
                self.registers.write_u16(Register16::Ax, value as u16);
            }
            Op::Mov { dst, src } => match (dst, src) {
                (Operand::Register16(reg1), Operand::Register16(reg2)) => {
                    let v = self.registers.read_u16(reg2);
                    self.registers.write_u16(reg1, v);
                }
                (Operand::Register16(reg1), Operand::MemoryBx) => {
                    let addr = self.registers.read_u16(Register16::Bx);
                    let v = machine.memory.read_u16(addr);
                    self.registers.write_u16(reg1, v);
                }
                (Operand::Register8(reg1), Operand::MemoryBx) => {
                    let addr = self.registers.read_u16(Register16::Bx);
                    let v = machine.memory.read_u8(addr);
                    self.registers.write_u8(reg1, v);
                }
                _ => panic!("Invalid combination"),
            },
            Op::Int(int) => match int {
                0x10 => {
                    print!("{}", self.registers.read_u8(Register8::Al) as char);
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
            0x8D => Op::Lea {
                addr: machine.read_u16(self),
            },
            0x8A => {
                let modrm: ModRm = machine.read_u8(self).into();
                let dst = Register8::from(modrm.reg).into();
                match modrm.mode {
                    0x03 => Op::Mov {
                        src: Register8::from(modrm.rm).into(),
                        dst,
                    },
                    0x00 => Op::Mov {
                        src: Operand::MemoryBx,
                        dst,
                    },
                    _ => panic!("Unhandled mode"),
                }
            }
            0x8B => {
                let modrm: ModRm = machine.read_u8(self).into();
                let dst = Register16::from(modrm.reg).into();
                match modrm.mode {
                    0x03 => Op::Mov {
                        src: Register16::from(modrm.rm).into(),
                        dst,
                    },
                    0x00 => Op::Mov {
                        src: Operand::MemoryBx,
                        dst,
                    },
                    _ => panic!("Unhandled mode"),
                }
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
                addr: machine.read_u16(self),
            },
            0x51 => Op::Jz {
                addr: machine.read_u16(self),
            },
            0x52 => Op::Jmp {
                addr: machine.read_u16(self),
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
            0x84 => {
                let modrm: ModRm = machine.read_u8(self).into();
                match modrm.mode {
                    0x03 => Op::Test {
                        op1: Register8::from(modrm.reg).into(),
                        op2: Register8::from(modrm.rm).into(),
                    },
                    _ => panic!("Invalid mod"),
                }
            }
            0x85 => {
                let modrm: ModRm = machine.read_u8(self).into();
                match modrm.mode {
                    0x03 => Op::Test {
                        op1: Register16::from(modrm.reg).into(),
                        op2: Register16::from(modrm.rm).into(),
                    },
                    _ => panic!("Invalid mod: {:?}", modrm),
                }
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
        Self {
            memory: [0; u16::MAX as usize],
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }
    pub fn read_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.memory[addr as usize], self.memory[addr as usize + 1]])
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

    pub fn read_u16(&mut self, cpu: &mut Cpu) -> u16 {
        let l = self.read_u8(cpu);
        let h = self.read_u8(cpu);

        u16::from_le_bytes([l, h])
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
