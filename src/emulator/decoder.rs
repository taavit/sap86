use std::{fs::File, io::Write};

use crate::{
    emulator::{cpu::Cpu, machine::Machine},
    isa::{
        EffectiveAddressBase, MemSpec, ModRm, Operand,
        instructions::{FarTarget, Op},
        registers::{Register8, Register16, SegmentRegister},
    },
};

pub fn fetch_decode(cpu: &mut Cpu, machine: &mut Machine) -> Op {
    let mut v;
    // Detects segment prefix
    let mut override_segment = None;
    let mut rep = false;
    loop {
        v = cpu.fetch_u8(machine);
        match v {
            0x26 => override_segment = Some(SegmentRegister::Es),
            0x36 => override_segment = Some(SegmentRegister::Ss),
            0x2E => override_segment = Some(SegmentRegister::Cs),
            0x3E => override_segment = Some(SegmentRegister::Ds),
            0xF3 => {
                rep = true;
            }
            _ => break,
        }
    }

    match v {
        0x90 => Op::Nop,
        0xF8 => Op::Clc,
        0xF9 => Op::Stc,
        0xFA => Op::Cli,
        0xFB => Op::Sti,
        0xFC => Op::Cld,
        0xFD => Op::Std,
        0xAC => Op::Lodsb {
            rep,
            override_segment,
        },
        0xE8 => {
            let offset = cpu.fetch_i16(machine);
            Op::Call {
                addr: Operand::RelAddress16(offset),
            }
        }
        0x8F => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);
            match modrm.reg {
                0 => Op::Pop { dst },
                _ => panic!("Invalid operand."),
            }
        }
        0xFF => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);
            match modrm.reg {
                0b000 => Op::Inc { dst },
                0b001 => Op::Dec { dst },
                0b010 => Op::Call { addr: dst },
                0b100 => Op::Jmp { addr: dst },
                0b101 => Op::JmpFar {
                    target: FarTarget::Indirect { ptr: dst },
                },
                0b110 => Op::Push { src: dst },
                r => panic!("Invalid reg 0xFF /{r}"),
            }
        }
        0xD1 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);
            let src = Operand::Imm16(1);
            match modrm.reg {
                0b000 => Op::Rol { dst, src },
                0b001 => Op::Ror { dst, src },
                0b010 => Op::Rcl { dst, src },
                0b011 => Op::Rcr { dst, src },
                0b100 => Op::Shl { dst, src },
                0b101 => Op::Shr { dst, src },
                _ => unimplemented!(),
            }
        }
        0xD2 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let dst = decode_rm8(cpu, machine, &modrm, override_segment);
            let src = Operand::Register8(Register8::Cl);
            match modrm.reg {
                0b000 => Op::Rol { dst, src },
                0b001 => Op::Ror { dst, src },
                0b010 => Op::Rcl { dst, src },
                0b011 => Op::Rcr { dst, src },
                0b100 => Op::Shl { dst, src },
                0b101 => Op::Shr { dst, src },
                _ => unimplemented!(),
            }
        }
        0xD3 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);
            let src = Operand::Register8(Register8::Cl);
            match modrm.reg {
                0b000 => Op::Rol { dst, src },
                0b001 => Op::Ror { dst, src },
                0b010 => Op::Rcl { dst, src },
                0b011 => Op::Rcr { dst, src },
                0b100 => Op::Shl { dst, src },
                0b101 => Op::Shr { dst, src },
                _ => unimplemented!(),
            }
        }
        0x0A => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let src = decode_rm8(cpu, machine, &modrm, override_segment);
            let dst = Operand::Register8(Register8::from(modrm.reg));
            Op::Or { src, dst }
        }
        0xC3 => Op::Ret,
        0xD0 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let src = Operand::Imm8(1);
            match modrm.reg {
                0b000 => Op::Rol {
                    dst: decode_rm8(cpu, machine, &modrm, override_segment),
                    src,
                },
                0b101 => Op::Shr {
                    dst: decode_rm8(cpu, machine, &modrm, override_segment),
                    src,
                },
                _ => unimplemented!("Missing: {}", modrm.reg),
            }
        }
        0xFE => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            match modrm.reg {
                0b000 => Op::Inc {
                    dst: decode_rm8(cpu, machine, &modrm, override_segment),
                },
                0b001 => Op::Dec {
                    dst: decode_rm8(cpu, machine, &modrm, override_segment),
                },
                _ => unreachable!("Invalid reg: {}", modrm.reg),
            }
        }
        0x24 => Op::And {
            src: Operand::Imm8(cpu.fetch_u8(machine)),
            dst: Operand::Register8(Register8::Al),
        },
        0x25 => Op::And {
            src: Operand::Imm16(cpu.fetch_u16(machine)),
            dst: Operand::Register16(Register16::Ax),
        },
        0x90..=0x97 => Op::Xchg {
            dst: Operand::Register16(Register16::from(v & 7)),
            src: Operand::Register16(Register16::Ax),
        },
        0x80 => {
            let (reg, dst) = cpu.fetch_rm8(machine, override_segment);
            let src = Operand::Imm8(cpu.fetch_u8(machine));
            match reg {
                0b000 => Op::Add { src, dst },
                0b001 => Op::Or { src, dst },
                0b010 => Op::Adc { src, dst },
                0b011 => Op::Sbb { src, dst },
                0b100 => Op::And { src, dst },
                0b101 => Op::Sub { src, dst },
                0b110 => Op::Xor { src, dst },
                0b111 => Op::Cmp { src, dst },
                _ => unimplemented!(),
            }
        }
        0x81 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);
            let src = Operand::Imm16(cpu.fetch_u16(machine));
            match modrm.reg {
                0b000 => Op::Add { src, dst },
                0b001 => Op::Or { src, dst },
                0b010 => Op::Adc { src, dst },
                0b011 => Op::Sbb { src, dst },
                0b100 => Op::And { src, dst },
                0b101 => Op::Sub { src, dst },
                0b110 => Op::Xor { src, dst },
                0b111 => Op::Cmp { src, dst },
                _ => unimplemented!(),
            }
        }
        0x83 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);
            let src = Operand::Imm8(cpu.fetch_u8(machine));
            match modrm.reg {
                0b000 => Op::Add { src, dst },
                0b010 => Op::Adc { src, dst },
                0b011 => Op::Sbb { src, dst },
                0b111 => Op::Cmp { src, dst },
                _ => panic!("Unhandled {}", modrm.reg),
            }
        }
        0x11 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let src = Operand::Register16(Register16::from(modrm.reg));
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);
            Op::Adc { src, dst }
        }
        0x13 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let dst = Operand::Register16(Register16::from(modrm.reg));
            let src = decode_rm16(cpu, machine, &modrm, override_segment);
            Op::Adc { src, dst }
        }
        0x00 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let src = Operand::Register8(Register8::from(modrm.reg));
            let dst = decode_rm8(cpu, machine, &modrm, override_segment);
            Op::Add { src, dst }
        }
        0x01 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let src = Operand::Register16(Register16::from(modrm.reg));
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);
            Op::Add { src, dst }
        }
        0x03 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let dst = Operand::Register16(Register16::from(modrm.reg));
            let src = decode_rm16(cpu, machine, &modrm, override_segment);
            Op::Add { src, dst }
        }
        0x04 => {
            let imm8 = cpu.fetch_u8(machine);
            Op::Add {
                src: Operand::Imm8(imm8),
                dst: Operand::Register8(Register8::Al),
            }
        }
        0x05 => {
            let imm16 = cpu.fetch_u16(machine);
            Op::Add {
                src: Operand::Imm16(imm16),
                dst: Operand::Register16(Register16::Ax),
            }
        }
        0x06 => Op::Push {
            src: Operand::SegmentRegister(SegmentRegister::Es),
        },
        0x07 => Op::Pop {
            dst: Operand::SegmentRegister(SegmentRegister::Es),
        },
        0x0E => Op::Push {
            src: Operand::SegmentRegister(SegmentRegister::Cs),
        },
        0x2B => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let dst = Operand::Register16(Register16::from(modrm.reg));
            let src = decode_rm16(cpu, machine, &modrm, override_segment);
            Op::Sub { src, dst }
        }
        0x2D => {
            let imm = cpu.fetch_u16(machine);
            let dst = Operand::Register16(Register16::Ax);
            let src = Operand::Imm16(imm);
            Op::Sub { src, dst }
        }
        0x29 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let src = Operand::Register16(Register16::from(modrm.reg));
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);
            Op::Sub { src, dst }
        }
        0x3D => {
            let imm = cpu.fetch_u16(machine);
            let dst = Operand::Register16(Register16::Ax);
            let src = Operand::Imm16(imm);
            Op::Cmp { src, dst }
        }
        0x40..=0x47 => Op::Inc {
            dst: Operand::Register16(Register16::from(v & 7)),
        },
        0x48..=0x4F => Op::Dec {
            dst: Operand::Register16(Register16::from(v & 7)),
        },
        0x8D => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);

            let dst = Operand::Register16(Register16::from(modrm.reg));
            let src = decode_rm16(cpu, machine, &modrm, override_segment);
            Op::Lea { src, dst }
        }
        0x3A => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            Op::Cmp {
                dst: Operand::Register8(Register8::from(modrm.reg)),
                src: decode_rm8(cpu, machine, &modrm, override_segment),
            }
        }
        0x3B => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            Op::Cmp {
                dst: Operand::Register16(Register16::from(modrm.reg)),
                src: decode_rm16(cpu, machine, &modrm, override_segment),
            }
        }
        0xC6 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            assert!(modrm.reg == 0);
            let dst = decode_rm8(cpu, machine, &modrm, override_segment);
            let imm = cpu.fetch_u8(machine);
            let src = Operand::Imm8(imm);
            Op::Mov { src, dst }
        }
        0x88 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);

            let dst = decode_rm8(cpu, machine, &modrm, override_segment);
            let src = Operand::Register8(Register8::from(modrm.reg));
            Op::Mov { src, dst }
        }
        0x8A => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);

            let dst = Operand::Register8(Register8::from(modrm.reg));
            let src = decode_rm8(cpu, machine, &modrm, override_segment);
            Op::Mov { src, dst }
        }
        0x8B => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);

            let dst = Operand::Register16(Register16::from(modrm.reg));
            let src = decode_rm16(cpu, machine, &modrm, override_segment);
            Op::Mov { src, dst }
        }
        0xB0..=0xB7 => {
            let imm = cpu.fetch_u8(machine);
            let reg = Register8::from(v & 7);
            Op::Mov {
                src: Operand::Imm8(imm),
                dst: reg.into(),
            }
        }
        0xB8..=0xBF => {
            let imm = cpu.fetch_u16(machine);
            let reg = Register16::from(v & 7);
            Op::Mov {
                src: Operand::Imm16(imm),
                dst: reg.into(),
            }
        }
        0xCD => Op::Int(cpu.fetch_u8(machine)),
        0xCC => Op::Int(0x03),
        0x84 => {
            let modrm: ModRm = cpu.fetch_u8(machine).into();
            match modrm.mode {
                0x03 => Op::Test {
                    op1: Register8::from(modrm.reg).into(),
                    op2: Register8::from(modrm.rm).into(),
                },
                _ => panic!("Invalid mod"),
            }
        }
        0x85 => {
            let modrm: ModRm = cpu.fetch_u8(machine).into();
            match modrm.mode {
                0x03 => Op::Test {
                    op1: Register16::from(modrm.reg).into(),
                    op2: Register16::from(modrm.rm).into(),
                },
                _ => panic!("Invalid mod: {:?}", modrm),
            }
        }
        0x89 => {
            let modrm = ModRm::from(cpu.fetch_u8(machine));
            let src = Operand::Register16(Register16::from(modrm.reg));
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);

            Op::Mov { src, dst }
        }
        0x8C => {
            let modrm = ModRm::from(cpu.fetch_u8(machine));
            let src = Operand::SegmentRegister(SegmentRegister::from(modrm.reg));
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);

            Op::Mov { src, dst }
        }
        0x8E => {
            let modrm = ModRm::from(cpu.fetch_u8(machine));
            let dst = Operand::SegmentRegister(SegmentRegister::from(modrm.reg));
            let src = decode_rm16(cpu, machine, &modrm, override_segment);

            Op::Mov { src, dst }
        }
        0xF4 => Op::Halt,
        0x74 => Op::Jz {
            addr: Operand::RelAddress16((cpu.fetch_u8(machine) as i8) as i16),
        },
        0x75 => Op::Jnz {
            addr: Operand::RelAddress16((cpu.fetch_u8(machine) as i8) as i16),
        },
        0x72 => Op::Jc {
            addr: Operand::RelAddress16((cpu.fetch_u8(machine) as i8) as i16),
        },
        0x7F => Op::Jg {
            addr: Operand::RelAddress16((cpu.fetch_u8(machine) as i8) as i16),
        },
        0x30 => {
            let modrm = ModRm::from(cpu.fetch_u8(machine));

            let src = Operand::Register8(Register8::from(modrm.reg));
            let dst = decode_rm8(cpu, machine, &modrm, override_segment);

            Op::Xor { src, dst }
        }
        0x31 => {
            let modrm = ModRm::from(cpu.fetch_u8(machine));

            let src = Operand::Register16(Register16::from(modrm.reg));
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);

            Op::Xor { src, dst }
        }
        0x33 => {
            let modrm = ModRm::from(cpu.fetch_u8(machine));

            let dst = Operand::Register16(Register16::from(modrm.reg));
            let src = decode_rm16(cpu, machine, &modrm, override_segment);

            Op::Xor { src, dst }
        }
        0x73 => Op::Jnc {
            addr: Operand::RelAddress16((cpu.fetch_u8(machine) as i8) as i16),
        },
        0xEB => Op::Jmp {
            addr: Operand::RelAddress16((cpu.fetch_u8(machine) as i8) as i16),
        },
        0xE9 => Op::Jmp {
            addr: Operand::RelAddress16(cpu.fetch_i16(machine)),
        },
        0xC7 => {
            let modrm = ModRm::from(cpu.fetch_u8(machine));
            match modrm.reg {
                0b000 => {
                    let dst = decode_rm16(cpu, machine, &modrm, override_segment);
                    let src = Operand::Imm16(cpu.fetch_u16(machine));
                    Op::Mov { src, dst }
                }
                _ => panic!("Unsupported reg: {}", modrm.reg),
            }
        }
        0x50..=0x57 => Op::Push {
            src: Operand::Register16(Register16::from(v & 7)),
        },
        0x32 => {
            let modrm = ModRm::from(cpu.fetch_u8(machine));
            let src = decode_rm8(cpu, machine, &modrm, override_segment);
            let dst = Operand::Register8(Register8::from(modrm.reg));

            Op::Xor { src, dst }
        }
        0x58..=0x5F => {
            let dst = Operand::Register16(Register16::from(v & 7));
            Op::Pop { dst }
        }
        0x86 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            Op::Xchg {
                dst: decode_rm8(cpu, machine, &modrm, override_segment),
                src: Operand::Register8(Register8::from(modrm.reg)),
            }
        }
        0x87 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            Op::Xchg {
                dst: decode_rm16(cpu, machine, &modrm, override_segment),
                src: Operand::Register16(Register16::from(modrm.reg)),
            }
        }
        0x16 => Op::Push {
            src: Operand::SegmentRegister(SegmentRegister::Ss),
        },
        0x1E => Op::Push {
            src: Operand::SegmentRegister(SegmentRegister::Ds),
        },
        0x1F => Op::Pop {
            dst: Operand::SegmentRegister(SegmentRegister::Ds),
        },
        0xA0 => {
            let dst = Operand::Register8(Register8::Al);
            let src = Operand::Mem8(MemSpec {
                base: EffectiveAddressBase::None,
                disp: cpu.fetch_i16(machine),
                is_direct: true,
                override_segment,
            });
            Op::Mov { dst, src }
        }
        0xA1 => {
            let dst = Operand::Register16(Register16::Ax);
            let src = Operand::Mem16(MemSpec {
                base: EffectiveAddressBase::None,
                disp: cpu.fetch_i16(machine),
                is_direct: true,
                override_segment,
            });
            Op::Mov { dst, src }
        }
        0xA2 => {
            let src = Operand::Register8(Register8::Al);
            let dst = Operand::Mem8(MemSpec {
                base: EffectiveAddressBase::None,
                disp: cpu.fetch_i16(machine),
                is_direct: true,
                override_segment,
            });
            Op::Mov { dst, src }
        }
        0xA3 => {
            let src = Operand::Register16(Register16::Ax);
            let dst = Operand::Mem16(MemSpec {
                base: EffectiveAddressBase::None,
                disp: cpu.fetch_i16(machine),
                is_direct: true,
                override_segment,
            });
            Op::Mov { dst, src }
        }
        0xA4 => Op::MovSb {
            rep,
            segment_override: override_segment,
        },
        0xA5 => Op::MovSw {
            rep,
            segment_override: override_segment,
        },
        0xA6 => Op::Cmpsb { rep },
        0xEA => {
            let offset = cpu.fetch_u16(machine);
            let segment = cpu.fetch_u16(machine);
            Op::JmpFar {
                target: FarTarget::Direct { segment, offset },
            }
        }
        0xCB => Op::RetFar,
        0xF6 => {
            let modrm = ModRm::from(cpu.fetch_u8(machine));
            match modrm.reg {
                0b000 => {
                    let op1 = decode_rm8(cpu, machine, &modrm, override_segment);
                    let imm = cpu.fetch_u8(machine);
                    Op::Test {
                        op1,
                        op2: Operand::Imm8(imm),
                    }
                }
                0b010 => {
                    let dst = decode_rm8(cpu, machine, &modrm, override_segment);
                    Op::Not { dst }
                }
                0b011 => {
                    let dst = decode_rm8(cpu, machine, &modrm, override_segment);
                    Op::Neg { dst }
                }
                0b100 => {
                    let src = decode_rm8(cpu, machine, &modrm, override_segment);
                    Op::Mul { src }
                }
                0b101 => {
                    let src = decode_rm8(cpu, machine, &modrm, override_segment);
                    Op::IMul { src }
                }
                0b110 => {
                    let src = decode_rm8(cpu, machine, &modrm, override_segment);
                    Op::Div { src }
                }
                0b111 => {
                    let src = decode_rm8(cpu, machine, &modrm, override_segment);
                    Op::IDiv { src }
                }
                _ => panic!("Unhandled mode: {}", modrm.reg),
            }
        }
        0xF7 => {
            let modrm = ModRm::from(cpu.fetch_u8(machine));
            match modrm.reg {
                0b000 => {
                    let op1 = decode_rm16(cpu, machine, &modrm, override_segment);
                    let imm = cpu.fetch_u16(machine);
                    Op::Test {
                        op1,
                        op2: Operand::Imm16(imm),
                    }
                }
                0b010 => {
                    let dst = decode_rm16(cpu, machine, &modrm, override_segment);
                    Op::Not { dst }
                }
                0b011 => {
                    let dst = decode_rm16(cpu, machine, &modrm, override_segment);
                    Op::Neg { dst }
                }
                0b100 => {
                    let src = decode_rm16(cpu, machine, &modrm, override_segment);
                    Op::Mul { src }
                }
                0b101 => {
                    let src = decode_rm16(cpu, machine, &modrm, override_segment);
                    Op::IMul { src }
                }
                0b110 => {
                    let src = decode_rm16(cpu, machine, &modrm, override_segment);
                    Op::Div { src }
                }
                0b111 => {
                    let src = decode_rm16(cpu, machine, &modrm, override_segment);
                    Op::IDiv { src }
                }
                _ => panic!("Unhandled mode: {}", modrm.reg),
            }
        }
        0x39 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            Op::Cmp {
                dst: decode_rm16(cpu, machine, &modrm, override_segment),
                src: Operand::Register16(Register16::from(modrm.reg)),
            }
        }
        0x3C => {
            let imm8 = cpu.fetch_u8(machine);
            Op::Cmp {
                dst: Operand::Register8(Register8::Al),
                src: Operand::Imm8(imm8),
            }
        }
        0xE6 => {
            let imm = Operand::Imm8(cpu.fetch_u8(machine));
            Op::Out {
                port: imm,
                value: Operand::Register8(Register8::Al),
            }
        }
        0xEE => Op::Out {
            port: Operand::Register16(Register16::Dx),
            value: Operand::Register8(Register8::Al),
        },
        0x98 => Op::Cbw,
        0xE0 => Op::LoopE {
            addr: Operand::RelAddress16(cpu.fetch_u8(machine) as i8 as i16),
        },
        0xE1 => Op::LoopNe {
            addr: Operand::RelAddress16(cpu.fetch_u8(machine) as i8 as i16),
        },
        0xE2 => Op::Loop {
            addr: Operand::RelAddress16(cpu.fetch_u8(machine) as i8 as i16),
        },
        0xC5 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            Op::Lds {
                dst: Operand::Register16(Register16::from(modrm.reg)),
                src: decode_rm16(cpu, machine, &modrm, override_segment),
            }
        }
        i => {
            let mem = machine.memory.dump();
            let mut f = File::create("dump.bin").unwrap();
            f.write_all(mem).unwrap();
            panic!("Unknown command: 0x{i:02X}, cpu: {cpu:?}")
        }
    }
}

pub fn decode_rm8(
    cpu: &mut Cpu,
    machine: &mut Machine,
    modrm: &ModRm,
    override_segment: Option<SegmentRegister>,
) -> Operand {
    match (modrm.mode, modrm.rm) {
        (0b00, 6) => {
            let addr = cpu.fetch_u16(machine);
            Operand::Mem8(MemSpec {
                base: EffectiveAddressBase::None,
                disp: addr as i16,
                is_direct: true,
                override_segment,
            })
        }
        (0b00, _) => Operand::Mem8(MemSpec {
            base: EffectiveAddressBase::from(modrm.rm),
            disp: 0,
            is_direct: false,
            override_segment,
        }),
        (0b01, _) => {
            let disp = cpu.fetch_u8(machine) as i8;
            Operand::Mem8(MemSpec {
                base: EffectiveAddressBase::from(modrm.rm),
                disp: disp as i16,
                is_direct: false,
                override_segment,
            })
        }
        (0b10, _) => {
            let disp = cpu.fetch_i16(machine);
            Operand::Mem8(MemSpec {
                base: EffectiveAddressBase::from(modrm.rm),
                is_direct: false,
                disp,
                override_segment,
            })
        }
        (0b11, _) => Operand::Register8(Register8::from(modrm.rm)),
        _ => unreachable!(),
    }
}

fn decode_rm16(
    cpu: &mut Cpu,
    machine: &mut Machine,
    modrm: &ModRm,
    override_segment: Option<SegmentRegister>,
) -> Operand {
    match modrm.mode {
        0b11 => Operand::Register16(Register16::from(modrm.rm)),
        _ => {
            if let Operand::Mem8(m) = decode_rm8(cpu, machine, modrm, override_segment) {
                Operand::Mem16(m)
            } else {
                unreachable!()
            }
        }
    }
}
