use std::{
    collections::HashMap,
    io::{self, Write},
};

use crate::registers::{Register8, Register16};

#[derive(Debug, Default)]
struct LabelMatch {
    target: Option<u16>,
    places: Vec<u16>,
}

#[derive(Debug)]
pub enum Operand {
    Register16(Register16),
    Register8(Register8),
    MemoryBx,
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

fn parse_register(r: &str) -> Register16 {
    match r.trim() {
        "ax" => Register16::Ax,
        "cx" => Register16::Cx,
        "dx" => Register16::Dx,
        "bx" => Register16::Bx,
        "sp" => Register16::Sp,
        "bp" => Register16::Bp,
        "si" => Register16::Si,
        "di" => Register16::Di,
        _ => panic!("Invalid register"),
    }
}

fn parse_operand(r: &str) -> Operand {
    match r.trim() {
        "ax" => Register16::Ax.into(),
        "cx" => Register16::Cx.into(),
        "dx" => Register16::Dx.into(),
        "bx" => Register16::Bx.into(),
        "sp" => Register16::Sp.into(),
        "bp" => Register16::Bp.into(),
        "si" => Register16::Si.into(),
        "di" => Register16::Di.into(),

        "ah" => Register8::Ah.into(),
        "al" => Register8::Al.into(),
        "ch" => Register8::Ch.into(),
        "cl" => Register8::Cl.into(),
        "dh" => Register8::Dh.into(),
        "dl" => Register8::Dl.into(),
        "bh" => Register8::Bh.into(),
        "bl" => Register8::Bl.into(),
        "[bx]" => Operand::MemoryBx,
        _ => panic!("Unknown operand"),
    }
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
        address: u16,
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
            Self::Lea { address } => {
                let splited = address.to_le_bytes();
                writer.write(&[0x8D, splited[0], splited[1]])
            }
            Self::Mov { dest, src } => {
                match (dest, src) {
                    (Operand::Register16(reg1), Operand::Register16(reg2)) => {
                        writer.write(&[0x8B, ModRm::new(3, *reg1 as u8, *reg2 as u8).into()])
                    }
                    (Operand::Register16(reg1), Operand::MemoryBx) => {
                        writer.write(&[0x8B, ModRm::new(0, *reg1 as u8, 7).into()])
                    }
                    _ => panic!("Invalid combination"),
                }
            }
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
                        Instruction::Lea { address: 0 }.write(&mut res).unwrap();
                        let entry = labels.entry(dest.trim().to_string()).or_default();
                        entry.places.push(res.len() as u16 - 2);
                    }
                    "mov" => {
                        let Some(RawToken::Token(reg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let Some(RawToken::Token(reg2)) = parsed.next() else {
                            panic!("Missing argument 2");
                        };
                        let dest = parse_operand(&reg1);
                        let src = parse_operand(&reg2);
                        Instruction::Mov { dest, src }.write(&mut res).unwrap();
                    }
                    "ld" => {
                        let Some(RawToken::Token(reg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let dst = parse_register(&reg1);
                        res.push(0x60 + dst as u8)
                    }
                    "test" => {
                        let Some(RawToken::Token(reg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let Some(RawToken::Token(reg2)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let reg1 = parse_operand(&reg1);
                        let reg2 = parse_operand(&reg2);
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
                        let dst = parse_register(&arg1);
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
                    "ldi" => {
                        let Some(RawToken::Token(arg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        res.push(0x10);
                        res.push(parse_argument(&arg1));
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

#[derive(Debug, PartialEq, Eq)]
enum ParserState {
    None,
    ReadToken,
    ReadString,
    ReadByte,
    Comment,
}

#[derive(Debug, PartialEq, Eq)]
enum RawToken {
    Token(String),
    StringToken(String),
    CharToken(String),
    EndLine,
}

fn tokenize<'a>(data: impl Iterator<Item = &'a u8>) -> Vec<RawToken> {
    let mut state = ParserState::None;
    let mut raw_tokens: Vec<RawToken> = Vec::new();
    let mut current_token = String::new();
    for byte in data {
        if state == ParserState::None {
            if *byte == b';' {
                state = ParserState::Comment;
                continue;
            }
            if *byte == b'\r' || *byte == b'\n' {
                state = ParserState::None;
                raw_tokens.push(RawToken::EndLine);
                continue;
            }
            if byte.is_ascii_whitespace() || *byte == b',' {
                continue;
            }
            if *byte == b'"' {
                state = ParserState::ReadString;
                continue;
            }
            if *byte == b'\'' {
                state = ParserState::ReadByte;
                continue;
            }
            state = ParserState::ReadToken;
            current_token.push(*byte as char);
        } else if state == ParserState::ReadToken {
            if *byte == b';' {
                state = ParserState::Comment;
                raw_tokens.push(RawToken::Token(current_token.clone()));
                current_token.clear();
                continue;
            }
            if *byte == b' ' || *byte == b'\t' {
                state = ParserState::None;
                raw_tokens.push(RawToken::Token(current_token.clone()));
                current_token.clear();
                continue;
            }
            if *byte == b',' {
                raw_tokens.push(RawToken::Token(current_token.clone()));
                current_token.clear();
                continue;
            }
            if *byte == b'\r' || *byte == b'\n' {
                state = ParserState::None;
                raw_tokens.push(RawToken::Token(current_token.clone()));
                raw_tokens.push(RawToken::EndLine);
                current_token.clear();
                continue;
            }
            current_token.push(*byte as char);
        } else if state == ParserState::Comment && (*byte == b'\r' || *byte == b'\n') {
            state = ParserState::None;
            raw_tokens.push(RawToken::EndLine);
            continue;
        } else if state == ParserState::ReadString {
            if *byte == b'"' {
                state = ParserState::None;
                raw_tokens.push(RawToken::StringToken(current_token.clone()));
                current_token.clear();
                continue;
            }
            current_token.push(*byte as char);
        } else if state == ParserState::ReadByte {
            if *byte == b'\'' {
                state = ParserState::None;
                raw_tokens.push(RawToken::CharToken(current_token.clone()));
                current_token.clear();
                continue;
            }
            current_token.push(*byte as char);
        }
    }
    if !current_token.is_empty() {
        raw_tokens.push(RawToken::Token(current_token));
    }

    raw_tokens
}

#[cfg(test)]
mod tests {
    use crate::compiler::{RawToken, tokenize};

    #[test]
    fn test_tokenizer() {
        let program = "
        ; Tokenizer test
        ; Test
        lea data  ; load binary data
        mov dx,ax ; store on data register
        data:
            db 'H'
            db \"ello\",\",\"
            db \" world!\",0

        ";
        let stream = program.as_bytes().iter();
        assert_eq!(
            tokenize(stream),
            vec![
                RawToken::EndLine,
                RawToken::EndLine,
                RawToken::EndLine,
                RawToken::Token("lea".to_string()),
                RawToken::Token("data".to_string()),
                RawToken::EndLine,
                RawToken::Token("mov".to_string()),
                RawToken::Token("dx".to_string()),
                RawToken::Token("ax".to_string()),
                RawToken::EndLine,
                RawToken::Token("data:".to_string()),
                RawToken::EndLine,
                RawToken::Token("db".to_string()),
                RawToken::CharToken("H".to_string()),
                RawToken::EndLine,
                RawToken::Token("db".to_string()),
                RawToken::StringToken("ello".to_string()),
                RawToken::StringToken(",".to_string()),
                RawToken::EndLine,
                RawToken::Token("db".to_string()),
                RawToken::StringToken(" world!".to_string()),
                RawToken::Token("0".to_string()),
                RawToken::EndLine,
                RawToken::EndLine,
            ]
        );
    }
}
