use crate::{
    emulator::{cpu::Cpu, machine::Machine},
    isa::{
        EffectiveAddressBase, MemSpec, ModRm, Operand,
        instructions::Op,
        registers::{Register8, Register16},
    },
};

pub fn fetch_decode(cpu: &mut Cpu, machine: &mut Machine) -> Op {
    let v = cpu.fetch_u8(machine);
    match v {
        0x90 => Op::Nop,
        0xFA => Op::Cli,
        0xFB => Op::Sti,
        0x40..=0x47 => Op::Inc {
            dst: Operand::Register16(Register16::from(v & 7)),
        },
        0x8D => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);

            let dst = Operand::Register16(Register16::from(modrm.reg));
            let src = decode_rm16(cpu, machine, modrm);
            Op::Lea { src, dst }
        }
        0x8A => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);

            let dst = Operand::Register8(Register8::from(modrm.reg));
            let src = decode_rm8(cpu, machine, modrm);
            Op::Mov { src, dst }
        }
        0x8B => {
            let modrm = cpu.fetch_u8(machine);
            let modrm = ModRm::from(modrm);

            let dst = Operand::Register16(Register16::from(modrm.reg));
            let src = decode_rm16(cpu, machine, modrm);
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
        0xF4 => Op::Halt,
        0x74 => Op::Jz {
            addr: Operand::RelAddress((cpu.fetch_u8(machine) as i8) as i16),
        },
        0x75 => Op::Jnz {
            addr: Operand::RelAddress((cpu.fetch_u8(machine) as i8) as i16),
        },
        0xEB => Op::Jmp {
            addr: Operand::RelAddress((cpu.fetch_u8(machine) as i8) as i16),
        },
        i => panic!("Unkown command: {i:02X}"),
    }
}

fn decode_rm8(cpu: &mut Cpu, machine: &mut Machine, modrm: ModRm) -> Operand {
    match (modrm.mode, modrm.rm) {
        (0b00, 6) => {
            let addr = cpu.fetch_u16(machine);
            Operand::Mem8(MemSpec {
                base: EffectiveAddressBase::None,
                disp: addr as i16,
                is_direct: true,
            })
        }
        (0b00, _) => Operand::Mem8(MemSpec {
            base: EffectiveAddressBase::from(modrm.rm),
            disp: 0,
            is_direct: false,
        }),
        (0b01, _) => {
            let disp = cpu.fetch_u8(machine) as i8;
            Operand::Mem8(MemSpec {
                base: EffectiveAddressBase::from(modrm.rm),
                disp: disp as i16,
                is_direct: false,
            })
        }
        (0b10, _) => {
            let disp = cpu.fetch_u16(machine) as i16;
            Operand::Mem8(MemSpec {
                base: EffectiveAddressBase::from(modrm.rm),
                is_direct: false,
                disp,
            })
        }
        (0b11, _) => Operand::Register8(Register8::from(modrm.rm)),
        _ => unreachable!(),
    }
}

fn decode_rm16(cpu: &mut Cpu, machine: &mut Machine, modrm: ModRm) -> Operand {
    match modrm.mode {
        0b11 => Operand::Register16(Register16::from(modrm.rm)),
        _ => {
            if let Operand::Mem8(m) = decode_rm8(cpu, machine, modrm) {
                Operand::Mem16(m)
            } else {
                unreachable!()
            }
        }
    }
}
