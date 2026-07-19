use crate::isa::registers::{Register8, Register16};

pub mod flags;
pub mod instructions;
pub mod registers;

#[derive(Debug)]
pub enum Operand {
    Register8(Register8),
    Register16(Register16),
    Imm16(u16),
    Imm8(u8),
    Mem8(MemSpec),
    Mem16(MemSpec),
    RelAddress(i16),
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

#[derive(Debug)]
pub struct ModRm {
    pub mode: u8,
    pub reg: u8,
    pub rm: u8,
}

impl From<u8> for ModRm {
    fn from(value: u8) -> Self {
        Self {
            mode: value >> 6,
            reg: (value >> 3) & 0x07,
            rm: value & 0x07,
        }
    }
}

#[derive(Debug)]
pub enum EffectiveAddressBase {
    BxSi,
    BxDi,
    BpSi,
    BpDi,
    Si,
    Di,
    Bp,
    Bx,
    None,
}

impl From<u8> for EffectiveAddressBase {
    fn from(value: u8) -> Self {
        match value & 7 {
            0 => Self::BxSi,
            1 => Self::BxDi,
            2 => Self::BpSi,
            3 => Self::BpDi,
            4 => Self::Si,
            5 => Self::Di,
            6 => Self::Bp,
            7 => Self::Bx,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct MemSpec {
    pub base: EffectiveAddressBase,
    /// Displacement
    pub disp: i16,
    pub is_direct: bool,
}
