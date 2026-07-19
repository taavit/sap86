use crate::{
    emulator::{bios::Bios, machine::Machine},
    isa::{
        EffectiveAddressBase, MemSpec, Operand,
        flags::Flags,
        instructions::Op,
        registers::{Register16, Registers, SegmentRegister},
    },
};

#[derive(Debug)]
pub struct Cpu {
    pub flags: Flags,
    pub registers: Registers,
    pub halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            flags: Flags {
                zero: false,
                interrupt: false,
            },
            registers: Registers::new(),
            halted: false,
        }
    }

    fn calculate_physical_address(segment: u16, offset: u16) -> u32 {
        ((segment as u32) << 4) + offset as u32
    }

    pub fn fetch_u8(&mut self, machine: &mut Machine) -> u8 {
        let address = Self::calculate_physical_address(
            self.registers.read_segment(SegmentRegister::Cs),
            self.registers.ip(),
        );
        let r = machine.read_physical_u8(address);
        self.registers.step_ip();

        r
    }

    pub fn fetch_u16(&mut self, machine: &mut Machine) -> u16 {
        let address = Self::calculate_physical_address(
            self.registers.read_segment(SegmentRegister::Cs),
            self.registers.ip(),
        );
        let r = machine.read_physical_u16(address);
        self.registers.step_ip_by(2);

        r
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
            Operand::Mem8(spec) => self.read_mem8(machine, spec) as u16,
            Operand::Mem16(spec) => self.read_mem16(machine, spec),
            Operand::SegmentRegister(reg) => self.registers.read_segment(*reg),
            _ => panic!("Invalid operand"),
        }
    }

    pub fn set_operand_value(&mut self, machine: &mut Machine, operand: &Operand, value: u16) {
        match operand {
            Operand::Register8(reg) => self.registers.write_u8(*reg, value as u8),
            Operand::Register16(reg) => self.registers.write_u16(*reg, value),
            Operand::Mem8(spec) => self.write_mem8(machine, spec, value as u8),
            Operand::Mem16(spec) => self.write_mem16(machine, spec, value),
            Operand::SegmentRegister(reg) => self.registers.write_segment(*reg, value),
            _ => panic!("Operand read only!"),
        }
    }

    pub fn execute(&mut self, machine: &mut Machine, instruction: Op) {
        match instruction {
            Op::Halt => {
                self.halted = true;
            }
            Op::Cli => {
                self.flags.interrupt = false;
            }
            Op::Sti => {
                self.flags.interrupt = true;
            }
            Op::Lea { src, dst } => {
                let addr = match src {
                    Operand::Mem8(spec) => self.resolve_address(&spec),
                    Operand::Mem16(spec) => self.resolve_address(&spec),
                    _ => panic!("Not address"),
                };
                self.set_operand_value(machine, &dst, addr);
            }
            Op::Jnz {
                addr: Operand::RelAddress(target),
            } => {
                if !self.flags.zero {
                    self.registers.set_ip(self.resolve_relative(target));
                }
            }
            Op::Sub { src, dst } => {
                let src_val = self.get_operand_value(machine, &src);
                let dst_val = self.get_operand_value(machine, &dst);
                let result = dst_val.wrapping_sub(src_val);
                self.flags.zero = result == 0;
                self.set_operand_value(machine, &dst, result);
            }
            Op::Jz {
                addr: Operand::RelAddress(target),
            } => {
                if self.flags.zero {
                    self.registers.set_ip(self.resolve_relative(target));
                }
            }
            Op::Inc { dst } => {
                let v = self.get_operand_value(machine, &dst);
                self.set_operand_value(machine, &dst, v.wrapping_add(1));
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
            Op::Mov { dst, src } => {
                let value = self.get_operand_value(machine, &src);
                self.set_operand_value(machine, &dst, value);
            }
            Op::Int(int) => Bios::handle_interrupt(int, self),
            Op::Nop => {}
            _ => panic!("Invalid instruction"),
        }
    }

    pub fn resolve_relative(&self, offset: i16) -> u16 {
        self.registers.ip().wrapping_add_signed(offset)
    }

    pub fn read_mem8(&self, machine: &Machine, spec: &MemSpec) -> u8 {
        let offset = self.resolve_address(spec);

        let segment = if spec.uses_bp() {
            SegmentRegister::Ss
        } else {
            SegmentRegister::Ds
        };

        machine.read_physical_u8(Self::calculate_physical_address(
            self.registers.read_segment(segment),
            offset,
        ))
    }

    pub fn read_mem16(&self, machine: &Machine, spec: &MemSpec) -> u16 {
        let offset = self.resolve_address(spec);

        let segment = if spec.uses_bp() {
            SegmentRegister::Ss
        } else {
            SegmentRegister::Ds
        };

        machine.read_physical_u16(Self::calculate_physical_address(
            self.registers.read_segment(segment),
            offset,
        ))
    }

    pub fn write_mem8(&self, machine: &mut Machine, spec: &MemSpec, value: u8) {
        let offset = self.resolve_address(spec);

        let segment = if spec.uses_bp() {
            SegmentRegister::Ss
        } else {
            SegmentRegister::Ds
        };

        machine.write_physical_u8(
            Self::calculate_physical_address(self.registers.read_segment(segment), offset),
            value,
        );
    }

    pub fn write_mem16(&self, machine: &mut Machine, spec: &MemSpec, value: u16) {
        let offset = self.resolve_address(spec);

        let segment = if spec.uses_bp() {
            SegmentRegister::Ss
        } else {
            SegmentRegister::Ds
        };

        machine.write_physical_u16(
            Self::calculate_physical_address(self.registers.read_segment(segment), offset),
            value,
        );
    }
}
