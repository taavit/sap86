use std::fmt::Display;

use crate::isa::registers::{Register8, Register16, SegmentRegister};

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
    SegmentRegister(SegmentRegister),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Register8(reg) => write!(f, "{reg}"),
            Operand::Register16(reg) => write!(f, "{reg}"),
            Operand::SegmentRegister(reg) => write!(f, "{reg}"),
            Operand::Imm8(v) => write!(f, "0x{v:02X}"),
            Operand::Imm16(v) => write!(f, "0x{v:04X}"),
            Operand::Mem16(s) => {
                if s.is_direct {
                    write!(f, "[{:04X}]", s.disp)
                } else {
                    match s.base {
                        EffectiveAddressBase::Bx => {
                            if s.disp == 0 {
                                write!(f, "[bx]")
                            } else {
                                write!(f, "[bx+0x{:04X}]", s.disp)
                            }
                        }
                        EffectiveAddressBase::Bp => {
                            if s.disp == 0 {
                                write!(f, "[bp]")
                            } else {
                                write!(f, "[bp+0x{:04X}]", s.disp)
                            }
                        }
                        EffectiveAddressBase::BxSi => {
                            if s.disp == 0 {
                                write!(f, "[bx+si]")
                            } else {
                                write!(f, "[bx+si+0x{:02X}]", s.disp)
                            }
                        }
                        EffectiveAddressBase::Di => {
                            if s.disp == 0 {
                                write!(f, "[di]")
                            } else {
                                write!(f, "[di+0x{:02X}]", s.disp)
                            }
                        }
                        _ => panic!("Not yet {:?}", s.base),
                    }
                }
            }
            Operand::Mem8(s) => {
                if s.is_direct {
                    write!(f, "[{:04X}]", s.disp)
                } else {
                    match s.base {
                        EffectiveAddressBase::Bx => {
                            if s.disp == 0 {
                                write!(f, "[bx]")
                            } else {
                                write!(f, "[bx+0x{:02X}]", s.disp)
                            }
                        }
                        EffectiveAddressBase::Bp => {
                            if s.disp == 0 {
                                write!(f, "[bp]")
                            } else {
                                write!(f, "[bp+0x{:02X}]", s.disp)
                            }
                        }
                        EffectiveAddressBase::Si => {
                            if s.disp == 0 {
                                write!(f, "[si]")
                            } else {
                                write!(f, "[si+0x{:02X}]", s.disp)
                            }
                        }
                        EffectiveAddressBase::BxSi => {
                            if s.disp == 0 {
                                write!(f, "[bx+si]")
                            } else {
                                write!(f, "[bx+si+0x{:02X}]", s.disp)
                            }
                        }
                        EffectiveAddressBase::Di => {
                            if s.disp == 0 {
                                write!(f, "[di]")
                            } else {
                                write!(f, "[di+0x{:02X}]", s.disp)
                            }
                        }
                        _ => panic!("Not yet {:?}", s.base),
                    }
                }
            }
            Operand::RelAddress(addr) => write!(f, "0x{addr:04X}"),
        }
    }
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

#[derive(Debug, PartialEq, Eq)]
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
    pub override_segment: Option<SegmentRegister>,
}

impl MemSpec {
    pub fn uses_bp(&self) -> bool {
        self.base == EffectiveAddressBase::Bp
            || self.base == EffectiveAddressBase::BpSi
            || self.base == EffectiveAddressBase::BpDi
    }
}
