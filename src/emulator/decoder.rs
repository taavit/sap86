use crate::{
    emulator::{cpu::Cpu, machine::Machine},
    isa::{
        EffectiveAddressBase, MemSpec, ModRm, Operand,
        instructions::Op,
        registers::{Register8, Register16, SegmentRegister},
    },
};

pub fn fetch_decode(cpu: &mut Cpu, machine: &mut Machine) -> Op {
    let mut v = cpu.fetch_u8(machine);
    // Detects segment prefix
    let mut override_segment = None;
    let mut rep = false;
    match v {
        0x26 => {
            v = cpu.fetch_u8(machine);
            override_segment = Some(SegmentRegister::Es)
        }
        0xF3 => {
            rep = true;
            v = cpu.fetch_u8(machine);
        }
        _ => {}
    }
    match v {
        0x90 => Op::Nop,
        0xFA => Op::Cli,
        0xFB => Op::Sti,
        0xFC => Op::Cld,
        0xFD => Op::Std,
        0xE8 => {
            let offset = cpu.fetch_u16(machine) as i16;
            Op::Call {
                addr: Operand::RelAddress(offset),
            }
        }
        0xC3 => Op::Ret,
        0x83 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let src = Operand::Imm8(cpu.fetch_u8(machine));
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);
            match modrm.reg {
                0b000 => Op::Add { src, dst },
                0b010 => Op::Adc { src, dst },
                _ => panic!("Unhandled {}", modrm.reg),
            }
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
        0x2B => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let dst = Operand::Register16(Register16::from(modrm.reg));
            let src = decode_rm16(cpu, machine, &modrm, override_segment);
            Op::Sub { src, dst }
        }
        0x29 => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);
            let src = Operand::Register16(Register16::from(modrm.reg));
            let dst = decode_rm16(cpu, machine, &modrm, override_segment);
            Op::Sub { src, dst }
        }
        0x40..=0x47 => Op::Inc {
            dst: Operand::Register16(Register16::from(v & 7)),
        },
        0x8D => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);

            let dst = Operand::Register16(Register16::from(modrm.reg));
            let src = decode_rm16(cpu, machine, &modrm, override_segment);
            Op::Lea { src, dst }
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
            addr: Operand::RelAddress((cpu.fetch_u8(machine) as i8) as i16),
        },
        0x75 => Op::Jnz {
            addr: Operand::RelAddress((cpu.fetch_u8(machine) as i8) as i16),
        },
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
        0xEB => Op::Jmp {
            addr: Operand::RelAddress((cpu.fetch_u8(machine) as i8) as i16),
        },
        0xE9 => Op::Jmp {
            addr: Operand::RelAddress(cpu.fetch_u16(machine) as i16),
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
        0xA1 => {
            let dst = Operand::Register16(Register16::Ax);
            let src = Operand::Mem16(MemSpec {
                base: EffectiveAddressBase::None,
                disp: cpu.fetch_u16(machine) as i16,
                is_direct: true,
                override_segment,
            });
            Op::Mov { dst, src }
        }
        0xA3 => {
            let src = Operand::Register16(Register16::Ax);
            let dst = Operand::Mem16(MemSpec {
                base: EffectiveAddressBase::None,
                disp: cpu.fetch_u16(machine) as i16,
                is_direct: true,
                override_segment,
            });
            Op::Mov { dst, src }
        }
        0xA4 => Op::MovSb { rep },
        0xA5 => Op::MovSw { rep },
        0xEA => {
            let offset = cpu.fetch_u16(machine);
            let segment = cpu.fetch_u16(machine);
            Op::JmpFar { segment, offset }
        }
        i => panic!("Unknown command: 0x{i:02X}, cpu: {cpu:?}"),
    }
}

fn decode_rm8(
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
            let disp = cpu.fetch_u16(machine) as i16;
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
