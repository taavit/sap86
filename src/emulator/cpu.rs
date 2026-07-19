use crate::{
    emulator::{bios::Bios, machine::Machine},
    isa::{
        EffectiveAddressBase, MemSpec, ModRm, Operand,
        flags::Flags,
        instructions::Op,
        registers::{Register8, Register16, Registers},
    },
};

pub struct Cpu {
    pub flags: Flags,
    pub registers: Registers,
    pub halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            flags: Flags { zero: false },
            registers: Registers::new(),
            halted: false,
        }
    }

    fn resolve_address(&self, spec: &MemSpec) -> u16 {
        if spec.is_direct {
            spec.disp as u16
        } else {
            let base_value = match spec.base {
                EffectiveAddressBase::BxSi => self
                    .registers
                    .read_u16(Register16::Bx)
                    .wrapping_add(self.registers.read_u16(Register16::Si)),
                EffectiveAddressBase::BxDi => self
                    .registers
                    .read_u16(Register16::Bx)
                    .wrapping_add(self.registers.read_u16(Register16::Di)),
                EffectiveAddressBase::BpSi => self
                    .registers
                    .read_u16(Register16::Bp)
                    .wrapping_add(self.registers.read_u16(Register16::Si)),
                EffectiveAddressBase::BpDi => self
                    .registers
                    .read_u16(Register16::Bp)
                    .wrapping_add(self.registers.read_u16(Register16::Di)),

                EffectiveAddressBase::Bx => self.registers.read_u16(Register16::Bx),
                EffectiveAddressBase::Di => self.registers.read_u16(Register16::Di),
                EffectiveAddressBase::Si => self.registers.read_u16(Register16::Si),
                EffectiveAddressBase::Bp => self.registers.read_u16(Register16::Bp),
                EffectiveAddressBase::None => 0,
            };
            base_value.wrapping_add(spec.disp as u16)
        }
    }

    pub fn get_operand_value(&self, machine: &mut Machine, operand: &Operand) -> u16 {
        match operand {
            Operand::Register8(reg) => self.registers.read_u8(*reg) as u16,
            Operand::Register16(reg) => self.registers.read_u16(*reg),
            Operand::Imm8(val) => *val as u16,
            Operand::Imm16(val) => *val,
            Operand::Mem8(spec) => machine.memory.read_u8(self.resolve_address(spec)) as u16,
            Operand::Mem16(spec) => machine.memory.read_u16(self.resolve_address(spec)),
            _ => panic!("Invalid operand"),
        }
    }

    pub fn set_operand_value(&mut self, machine: &mut Machine, operand: &Operand, value: u16) {
        match operand {
            Operand::Register8(reg) => self.registers.write_u8(*reg, value as u8),
            Operand::Register16(reg) => self.registers.write_u16(*reg, value),
            Operand::Mem8(spec) => machine
                .memory
                .write_u8(self.resolve_address(spec), value as u8),
            Operand::Mem16(spec) => machine.memory.write_u16(self.resolve_address(spec), value),
            _ => panic!("Operand read only!"),
        }
    }

    pub fn execute(&mut self, machine: &mut Machine, instruction: Op) {
        match instruction {
            Op::Halt => {
                self.halted = true;
            }
            Op::Lea { src, dst } => {
                let addr = match src {
                    Operand::Mem8(spec) => self.resolve_address(&spec),
                    Operand::Mem16(spec) => self.resolve_address(&spec),
                    _ => panic!("Not address"),
                };
                self.set_operand_value(machine, &dst, addr);
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
            Op::Jnz {
                addr: Operand::RelAddress(target),
            } => {
                if !self.flags.zero {
                    self.registers.set_ip(self.resolve_relative(target));
                }
            }
            Op::Jz {
                addr: Operand::RelAddress(target),
            } => {
                if self.flags.zero {
                    self.registers.set_ip(self.resolve_relative(target));
                }
            }
            Op::Jmp {
                addr: Operand::RelAddress(target),
            } => {
                self.registers.set_ip(self.resolve_relative(target));
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
            Op::Mov { dst, src } => {
                let value = self.get_operand_value(machine, &src);
                self.set_operand_value(machine, &dst, value);
            }
            Op::Int(int) => Bios::handle_interrupt(int, self),
            Op::Nop => {}
            _ => panic!("Invalid instruction"),
        }
    }

    pub fn fetch_decode(&mut self, machine: &mut Machine) -> Op {
        let v = machine.read_u8(self);
        match v {
            0x00 => Op::Nop,
            0x8D => {
                let modrm = machine.read_u8(self);
                let modrm = ModRm::from(modrm);

                let dst = Operand::Register16(Register16::from(modrm.reg));
                let src = decode_rm16(self, machine, modrm);
                Op::Lea { src, dst }
            }
            0x8A => {
                let modrm = machine.read_u8(self);
                let modrm = ModRm::from(modrm);

                let dst = Operand::Register8(Register8::from(modrm.reg));
                let src = decode_rm8(self, machine, modrm);
                Op::Mov { src, dst }
            }
            0x8B => {
                let modrm = machine.read_u8(self);
                let modrm = ModRm::from(modrm);

                let dst = Operand::Register16(Register16::from(modrm.reg));
                let src = decode_rm16(self, machine, modrm);
                Op::Mov { src, dst }
            }
            0xB0..=0xB7 => {
                let imm = machine.read_u8(self);
                let reg = Register8::from(v & 7);
                Op::Mov {
                    src: Operand::Imm8(imm),
                    dst: reg.into(),
                }
            }
            0xB8..=0xBF => {
                let imm = machine.read_u16(self);
                let reg = Register16::from(v & 7);
                Op::Mov {
                    src: Operand::Imm16(imm),
                    dst: reg.into(),
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
            0x74 => Op::Jz {
                addr: Operand::RelAddress(machine.read_rel8(self)),
            },
            0x75 => Op::Jnz {
                addr: Operand::RelAddress(machine.read_rel8(self)),
            },
            0xEB => Op::Jmp {
                addr: Operand::RelAddress(machine.read_rel8(self)),
            },
            i => panic!("Unkown command: {i:02X}"),
        }
    }

    pub fn resolve_relative(&self, offset: i16) -> u16 {
        self.registers.ip().wrapping_add_signed(offset)
    }
}

fn decode_rm8(cpu: &mut Cpu, machine: &mut Machine, modrm: ModRm) -> Operand {
    match (modrm.mode, modrm.rm) {
        (0b00, 6) => {
            let addr = machine.read_u16(cpu);
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
            let disp = machine.read_u8(cpu) as i8;
            Operand::Mem8(MemSpec {
                base: EffectiveAddressBase::from(modrm.rm),
                disp: disp as i16,
                is_direct: false,
            })
        }
        (0b10, _) => {
            let disp = machine.read_u16(cpu) as i16;
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
