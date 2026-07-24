use std::fmt::Display;

use crate::isa::{Operand, registers::SegmentRegister};

#[derive(Debug)]
pub enum FarTarget {
    Direct { segment: u16, offset: u16 },
    Indirect { ptr: Operand },
}

#[derive(Debug)]
pub enum Op {
    Nop,
    Cli,
    Sti,
    Std,
    Cld,
    Clc,
    Stc,
    Lodsb {
        rep: bool,
        override_segment: Option<SegmentRegister>,
    },
    Lea {
        src: Operand,
        dst: Operand,
    },
    Jnz {
        addr: Operand,
    },
    Jz {
        addr: Operand,
    },
    Jg {
        addr: Operand,
    },
    Jmp {
        addr: Operand,
    },
    Jnc {
        addr: Operand,
    },
    Jc {
        addr: Operand,
    },
    Inc {
        dst: Operand,
    },
    Dec {
        dst: Operand,
    },
    Not {
        dst: Operand,
    },
    Neg {
        dst: Operand,
    },
    Rol {
        dst: Operand,
        src: Operand,
    },
    Ror {
        dst: Operand,
        src: Operand,
    },
    Rcl {
        dst: Operand,
        src: Operand,
    },
    Rcr {
        dst: Operand,
        src: Operand,
    },
    Shl {
        dst: Operand,
        src: Operand,
    },
    Sal {
        dst: Operand,
        src: Operand,
    },
    Shr {
        dst: Operand,
        src: Operand,
    },
    Sar {
        dst: Operand,
        src: Operand,
    },
    Test {
        op1: Operand,
        op2: Operand,
    },
    Mov {
        src: Operand,
        dst: Operand,
    },
    Sub {
        src: Operand,
        dst: Operand,
    },
    Add {
        src: Operand,
        dst: Operand,
    },
    Or {
        src: Operand,
        dst: Operand,
    },
    And {
        src: Operand,
        dst: Operand,
    },
    Sbb {
        src: Operand,
        dst: Operand,
    },
    Adc {
        src: Operand,
        dst: Operand,
    },
    Mul {
        src: Operand,
    },
    IMul {
        src: Operand,
    },
    Div {
        src: Operand,
    },
    IDiv {
        src: Operand,
    },
    Xor {
        src: Operand,
        dst: Operand,
    },
    JmpFar {
        target: FarTarget,
    },
    Int(u8),
    RetFar,
    Call {
        addr: Operand,
    },
    MovSb {
        rep: bool,
        segment_override: Option<SegmentRegister>,
    },
    MovSw {
        rep: bool,
        segment_override: Option<SegmentRegister>,
    },
    Push {
        src: Operand,
    },
    Pop {
        dst: Operand,
    },
    Cmp {
        dst: Operand,
        src: Operand,
    },
    Xchg {
        dst: Operand,
        src: Operand,
    },
    Out {
        port: Operand,
        value: Operand,
    },
    Ret,
    Halt,
    Cbw,
    LoopNe {
        addr: Operand,
    },
    LoopE {
        addr: Operand,
    },
    Loop {
        addr: Operand,
    },
    Lds {
        dst: Operand,
        src: Operand,
    },
    Cmpsb {
        rep: bool,
    },
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Nop => write!(f, "nop"),
            Op::Cli => write!(f, "cli"),
            Op::Mov { dst, src } => write!(f, "mov {},{}", dst, src),
            Op::Xor { dst, src } => write!(f, "xor {},{}", dst, src),
            Op::MovSw {
                rep,
                segment_override,
            } => {
                if let Some(segment) = segment_override {
                    write!(f, "{segment}:")?;
                }
                f.write_str(if *rep { "rep movsw" } else { "movsw" })
            }
            Op::JmpFar { target } => match target {
                FarTarget::Direct { segment, offset } => {
                    write!(f, "jmpf {segment:04X}:{offset:04X}")
                }
                FarTarget::Indirect { ptr: dst } => write!(f, "jmpf {dst}"),
            },
            Op::Int(int) => write!(f, "int {int:02X}h"),
            Op::Call { addr } => write!(f, "call {addr}"),
            Op::Test { op1, op2 } => write!(f, "test {op1},{op2}"),
            Op::Cmp { dst, src } => write!(f, "cmp {dst},{src}"),
            Op::Jnz { addr } => write!(f, "jnz {addr}"),
            Op::Jz { addr } => write!(f, "jz {addr}"),
            Op::Jnc { addr } => write!(f, "jnc {addr}"),
            Op::Jc { addr } => write!(f, "jc {addr}"),
            Op::Inc { dst } => write!(f, "inc {dst}"),
            Op::Dec { dst } => write!(f, "dec {dst}"),
            Op::Push { src } => write!(f, "push {src}"),
            Op::Pop { dst } => write!(f, "pop {dst}"),
            Op::And { src, dst } => write!(f, "and {dst},{src}"),
            Op::Jmp { addr } => write!(f, "jmp {addr}"),
            Op::Div { src } => write!(f, "div {src}"),
            Op::Mul { src } => write!(f, "mul {src}"),
            Op::IDiv { src } => write!(f, "idiv {src}"),
            Op::Add { src, dst } => write!(f, "add {dst},{src}"),
            Op::Sub { src, dst } => write!(f, "sub {dst},{src}"),
            Op::Shl { dst, src } => write!(f, "shl {dst},{src}"),
            Op::Shr { dst, src } => write!(f, "shr {dst},{src}"),
            Op::Out { port, value } => write!(f, "out {port},{value}"),
            Op::Ret => write!(f, "ret"),
            Op::Cld => write!(f, "cld"),
            Op::Clc => write!(f, "clc"),
            Op::Lea { src, dst } => write!(f, "lea, {dst},{src}"),
            Op::Sti => write!(f, "sti"),
            Op::Adc { dst, src } => write!(f, "adc {dst},{src}"),
            Op::Lodsb {
                rep,
                override_segment,
            } => {
                if let Some(segment) = override_segment {
                    write!(f, "{segment}: ")?;
                }
                if *rep {
                    write!(f, "rep ")?;
                }
                write!(f, "lodsb")
            }
            Op::Cmpsb { rep } => {
                if *rep {
                    write!(f, "rep ")?;
                }
                write!(f, "cmpsb")
            }
            Op::Halt => write!(f, "hlt"),
            Op::Cbw => write!(f, "cbw"),
            Op::MovSb {
                rep,
                segment_override,
            } => {
                if let Some(segment) = segment_override {
                    write!(f, "{segment}: ")?;
                }
                f.write_str(if *rep { "rep movsb" } else { "movsb" })
            }
            Op::Or { src, dst } => write!(f, "or {dst},{src}"),
            Op::Jg { addr } => write!(f, "jg {addr}"),
            Op::LoopE { addr } => write!(f, "loope {addr}"),
            Op::LoopNe { addr } => write!(f, "loopne {addr}"),
            Op::Ror { dst, src } => write!(f, "ror {dst},{src}"),
            Op::Xchg { dst, src } => write!(f, "xchg {dst},{src}"),
            Op::Loop { addr } => write!(f, "loop {addr}"),
            Op::Lds { dst, src } => write!(f, "lds {dst},{src}"),
            Op::RetFar => write!(f, "retf"),
            Op::Sbb { src, dst } => write!(f, "sbb {dst},{src}"),
            _ => panic!("Not ready {:?}", self),
        }
    }
}
