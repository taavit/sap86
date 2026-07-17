use std::{
    collections::HashMap,
    io::{self, Write},
};

use crate::{
    compiler::tokenizer::{RawToken, tokenize}, isa::{MemorySpec, registers::{Register8, Register16}},
};

mod tokenizer;
mod memory_operand;

#[derive(Debug, Default)]
struct LabelMatch {
    target: Option<u16>,
    places: Vec<u16>,
}

#[derive(Debug)]
pub enum Operand {
    Register16(Register16),
    Register8(Register8),
    MemorySpec(MemorySpec),
    Imm8(u8),
    Imm16(u16),
}

impl From<Register8> for Operand {
    fn from(value: Register8) -> Self {
        Operand::Register8(value)
    }
}

impl From<Register16> for Operand {
    fn from(value: Register16) -> Self {
        Operand::Register16(value)
    }
}

fn parse_register16(r: &str) -> Option<Register16> {
    match r.trim() {
        "ax" => Some(Register16::Ax),
        "cx" => Some(Register16::Cx),
        "dx" => Some(Register16::Dx),
        "bx" => Some(Register16::Bx),
        "sp" => Some(Register16::Sp),
        "bp" => Some(Register16::Bp),
        "si" => Some(Register16::Si),
        "di" => Some(Register16::Di),
        _ => None,
    }
}

fn parse_register8(r: &str) -> Option<Register8> {
    match r.trim() {
        "al" => Some(Register8::Al),
        "cl" => Some(Register8::Cl),
        "dl" => Some(Register8::Dl),
        "bl" => Some(Register8::Bl),
        "ah" => Some(Register8::Ah),
        "ch" => Some(Register8::Ch),
        "dh" => Some(Register8::Dh),
        "bh" => Some(Register8::Bh),
        _ => None,
    }
}

fn parse_memory(r: &str) -> Option<Operand> {
    if r == "[bx]" {
        Some(Operand::MemorySpec(MemorySpec::MemoryBx))
    } else {
        None
    }
}

fn parse_number(r: &str) -> Option<Operand> {
    if let Some(v) = r.strip_suffix('h') {
        let v = u16::from_str_radix(v, 16).unwrap();
        if v > u8::MAX.into() {
            Some(Operand::Imm16(v))
        } else {
            Some(Operand::Imm8(v as u8))
        }
    } else if let Ok(v) = r.parse() {
        if v > u8::MAX.into() {
            Some(Operand::Imm16(v))
        } else {
            Some(Operand::Imm8(v as u8))
        }
    } else {
        None
    }
}

fn parse_operand(r: &str) -> Option<Operand> {
    parse_register8(r)
        .map(Operand::from)
        .or_else(|| parse_register16(r).map(Operand::from))
        .or_else(|| parse_memory(r))
        .or_else(|| parse_number(r))
}

fn parse_argument(arg: &str) -> u8 {
    let arg = arg.trim();
    if let Some(arg) = arg.strip_suffix('h') {
        u8::from_str_radix(arg, 16).unwrap()
    } else {
        arg.parse().unwrap()
    }
}

#[derive(Debug)]
pub struct ModRm {
    pub mode: u8,
    pub reg: u8,
    pub rm: u8,
}

impl ModRm {
    pub fn new(mode: u8, reg: u8, rm: u8) -> Self {
        Self { mode, reg, rm }
    }
}

impl From<u8> for ModRm {
    fn from(value: u8) -> Self {
        Self {
            mode: value >> 6,
            reg: (value >> 3) & 0x7,
            rm: value & 0x07,
        }
    }
}

impl From<ModRm> for u8 {
    fn from(value: ModRm) -> Self {
        (value.mode << 6) | (value.reg << 3) | value.rm
    }
}



pub enum Instruction {
    Halt,
    Int {
        interrupt: u8,
    },
    Lea {
        src: Operand,
        dest: Operand,
    },
    Mov {
        dest: Operand,
        src: Operand,
    },
    Test {
        operand1: Operand,
        operand2: Operand,
    },
}

impl Instruction {
    pub fn write(&self, mut writer: impl Write) -> Result<usize, io::Error> {
        match self {
            Self::Halt => writer.write(&[0xF4]),
            Self::Int { interrupt } => {
                if *interrupt == 3 {
                    writer.write(&[0xCC])
                } else {
                    writer.write(&[0xCD, *interrupt])
                }
            }
            Self::Lea { src, dest: Operand::Register16(reg) } => {
                let modrm = ModRm {
                    mode: 0,
                    reg: *reg as u8,
                    rm: 6
                };
                match src {
                    Operand::MemorySpec(MemorySpec::Displacement(address)) => {
                        let splited = address.to_le_bytes();
                        writer.write(&[0x8D, modrm.into(), splited[0], splited[1]])
                    }
                    _ => panic!("Invalid dest"),
                }
            }
            Self::Lea { src: _, dest: _ } => panic!("Invalid operands"),
            Self::Mov { dest, src } => match (dest, src) {
                (Operand::Register16(reg1), Operand::Register16(reg2)) => {
                    writer.write(&[0x8B, ModRm::new(3, *reg1 as u8, *reg2 as u8).into()])
                }
                (Operand::Register16(reg1), Operand::MemorySpec(MemorySpec::MemoryBx)) => {
                    writer.write(&[0x8B, ModRm::new(0, *reg1 as u8, 7).into()])
                }
                (Operand::Register8(reg1), Operand::MemorySpec(MemorySpec::MemoryBx)) => {
                    writer.write(&[0x8A, ModRm::new(0, *reg1 as u8, 7).into()])
                }
                (Operand::Register8(reg1), Operand::Imm8(v)) => {
                    writer.write(&[0xB0 + *reg1 as u8, *v])
                }
                (Operand::Register16(reg1), Operand::Imm16(v)) => {
                    let b = v.to_le_bytes();
                    writer.write(&[0xB8 + *reg1 as u8, b[0], b[1]])
                }
                (Operand::Register16(reg1), Operand::Imm8(v)) => {
                    let b = (*v as u16).to_le_bytes();
                    writer.write(&[0xB8 + *reg1 as u8, b[0], b[1]])
                }
                _ => panic!("Invalid combination"),
            },
            Self::Test { operand1, operand2 } => match (operand1, operand2) {
                (Operand::Register16(reg1), Operand::Register16(reg2)) => {
                    writer.write(&[0x85, ModRm::new(3, *reg1 as u8, *reg2 as u8).into()])
                }
                (Operand::Register8(reg1), Operand::Register8(reg2)) => {
                    writer.write(&[0x84, ModRm::new(3, *reg1 as u8, *reg2 as u8).into()])
                }
                _ => {
                    panic!("Invalid operands");
                }
            },
        }
    }
}

pub fn compile_program(program: &str) -> Vec<u8> {
    let mut res = Vec::new();
    let mut labels: HashMap<String, LabelMatch> = HashMap::new();
    let mut parsed = tokenize(program.as_bytes().iter()).into_iter();
    while let Some(next) = parsed.next() {
        match next {
            RawToken::EndLine => {
                continue;
            }
            RawToken::Token(token) => {
                if let Some(l) = token.strip_suffix(':') {
                    labels.entry(l.to_string()).or_default().target = Some(res.len() as u16);
                    continue;
                }
                match token.as_str() {
                    "lea" => {
                        let Some(RawToken::Token(dest)) = parsed.next() else {
                            panic!("Missing argument");
                        };
                        let Some(RawToken::Token(src_raw)) = parsed.next() else {
                            panic!("Missing argument");
                        };
                        let src = parse_operand(&src_raw);
                        let dest = parse_operand(&dest).unwrap();
                        let src = if let Some(op) = src {
                            op
                        } else {
                            let entry = labels.entry(src_raw).or_default();
                            entry.places.push(res.len() as u16 + 2);
                            Operand::MemorySpec(MemorySpec::Displacement(0))
                        };
                        Instruction::Lea { src, dest }.write(&mut res).unwrap();
                    }
                    "mov" => {
                        let Some(RawToken::Token(reg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let Some(RawToken::Token(reg2)) = parsed.next() else {
                            panic!("Missing argument 2");
                        };
                        let dest = parse_operand(&reg1).unwrap();
                        let src = parse_operand(&reg2).unwrap();
                        Instruction::Mov { dest, src }.write(&mut res).unwrap();
                    }
                    "ld" => {
                        let Some(RawToken::Token(reg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let dst = parse_register16(&reg1).unwrap();
                        res.push(0x60 + dst as u8)
                    }
                    "test" => {
                        let Some(RawToken::Token(reg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let Some(RawToken::Token(reg2)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let reg1 = parse_operand(&reg1).unwrap();
                        let reg2 = parse_operand(&reg2).unwrap();
                        Instruction::Test {
                            operand1: reg1,
                            operand2: reg2,
                        }
                        .write(&mut res)
                        .unwrap();
                    }
                    "jz" => {
                        let Some(RawToken::Token(arg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let entry = labels.entry(arg1).or_default();
                        res.push(0x51);
                        entry.places.push(res.len() as u16);
                        res.push(0);
                        res.push(0);
                    }
                    "int" => {
                        let Some(RawToken::Token(arg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let v = parse_argument(&arg1);
                        Instruction::Int { interrupt: v }.write(&mut res).unwrap();
                    }
                    "inc" => {
                        let Some(RawToken::Token(arg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let dst = parse_register16(&arg1).unwrap();
                        res.push(0x40 + dst as u8);
                    }
                    "jmp" => {
                        let Some(RawToken::Token(arg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        res.push(0x52);
                        let entry = labels.entry(arg1).or_default();
                        entry.places.push(res.len() as u16);
                        res.push(0);
                        res.push(0);
                    }
                    "hlt" => {
                        Instruction::Halt.write(&mut res).unwrap();
                    }
                    "db" => {
                        for db in parsed.by_ref() {
                            match db {
                                RawToken::CharToken(t) => res.push(*t.as_bytes().first().unwrap()),
                                RawToken::StringToken(t) => {
                                    for b in t.as_bytes() {
                                        res.push(*b);
                                    }
                                }
                                RawToken::Token(t) => {
                                    let c = parse_argument(&t);
                                    res.push(c);
                                }
                                _ => break,
                            }
                        }
                    }
                    _ => panic!("Unknown token {:?}", token),
                }
            }
            _ => panic!("Invalid token: {:?}", next),
        }
    }

    for (label, position) in labels.drain() {
        if let Some(target) = position.target {
            for place in position.places {
                res[place as usize] = (target & 0x00FF) as u8;
                res[place as usize + 1] = (target >> 8) as u8;
            }
        } else {
            panic!("Label {label} not found");
        }
    }

    res
}
