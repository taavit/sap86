use crate::{
    emulator::{
        instruction::{exec_adc, exec_add, exec_dec, exec_inc, exec_sub, exec_xor},
        machine::Machine,
    },
    isa::{
        EffectiveAddressBase, MemSpec, Operand,
        flags::Flags,
        instructions::Op,
        registers::{Register8, Register16, Registers, SegmentRegister},
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
            flags: Flags::default(),
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

    fn push_u16(&mut self, machine: &mut Machine, value: u16) {
        let offset = self.registers.read_u16(Register16::Sp).wrapping_sub(2);
        self.registers.write_u16(Register16::Sp, offset);
        let segment = self.registers.read_segment(SegmentRegister::Ss);
        let address = Self::calculate_physical_address(segment, offset);
        machine.write_physical_u16(address, value)
    }

    fn pop_u16(&mut self, machine: &Machine) -> u16 {
        let sp = self.registers.read_u16(Register16::Sp);
        let segment = self.registers.read_segment(SegmentRegister::Ss);
        let address = Self::calculate_physical_address(segment, sp);
        let v = machine.read_physical_u16(address);
        self.registers.write_u16(Register16::Sp, sp.wrapping_add(2));
        v
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
            Op::Cld => {
                self.flags.direction = false;
            }
            Op::Std => {
                self.flags.direction = true;
            }
            Op::Push { src } => {
                let v = self.get_operand_value(machine, &src);
                self.push_u16(machine, v);
            }
            Op::Pop { dst } => {
                let v = self.pop_u16(machine);
                self.set_operand_value(machine, &dst, v);
            }
            Op::Cbw => {
                let v = self.registers.read_u8(Register8::Al);
                self.registers.write_u16(Register16::Ax, v as u16);
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
            Op::Call {
                addr: Operand::RelAddress(target),
            } => {
                let ip = self.registers.ip();
                self.push_u16(machine, ip);
                self.registers.set_ip(self.resolve_relative(target));
            }
            Op::Ret => {
                let return_address = self.pop_u16(machine);
                self.registers.set_ip(return_address);
            }
            Op::Sub { src, dst } => exec_sub(&src, &dst, self, machine),
            Op::Add { src, dst } => exec_add(&src, &dst, self, machine),
            Op::Adc { src, dst } => exec_adc(&src, &dst, self, machine),
            Op::Xor { src, dst } => exec_xor(&src, &dst, self, machine),
            Op::And { src, dst } => {
                let src_val = self.get_operand_value(machine, &src);
                let dst_val = self.get_operand_value(machine, &dst);
                match dst {
                    Operand::Register8(_) | Operand::Mem8(_) => {
                        let dst_val = dst_val as u8;
                        let src_val = src_val as u8;
                        let result = dst_val & src_val;
                        self.flags.zero = result == 0;
                        self.flags.carry = false;
                        self.flags.sign = (result & 0x80) != 0;
                        self.flags.overflow = false;
                        self.flags.parity = result.count_ones().is_multiple_of(2);
                        self.set_operand_value(machine, &dst, result as u16);
                    }
                    Operand::Register16(_) | Operand::Mem16(_) => {
                        let result = dst_val & src_val;
                        self.flags.zero = result == 0;
                        self.flags.sign = (result & 0x8000) != 0;
                        self.flags.carry = false;
                        self.flags.overflow = false;
                        self.flags.parity = (result as u8).count_ones().is_multiple_of(2);
                        self.set_operand_value(machine, &dst, result);
                    }
                    _ => panic!("Invalid combination"),
                }
            }
            Op::Jz {
                addr: Operand::RelAddress(target),
            } => {
                if self.flags.zero {
                    self.registers.set_ip(self.resolve_relative(target));
                }
            }
            Op::Jg {
                addr: Operand::RelAddress(target),
            } => {
                if !self.flags.zero && self.flags.overflow == self.flags.sign {
                    self.registers.set_ip(self.resolve_relative(target));
                }
            }
            Op::Inc { dst } => exec_inc(&dst, self, machine),
            Op::Dec { dst } => exec_dec(&dst, self, machine),
            Op::Jmp {
                addr: Operand::RelAddress(target),
            } => {
                self.registers.set_ip(self.resolve_relative(target));
            }
            Op::Jnc {
                addr: Operand::RelAddress(target),
            } => {
                if !self.flags.carry {
                    self.registers.set_ip(self.resolve_relative(target));
                }
            }
            Op::Jc {
                addr: Operand::RelAddress(target),
            } => {
                if self.flags.carry {
                    self.registers.set_ip(self.resolve_relative(target));
                }
            }
            Op::Test { op1, op2 } => {
                match (op1, op2) {
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
                }
                self.flags.carry = false;
                self.flags.overflow = false;
            }
            Op::Cmp { dst, src } => {
                let src_val = self.get_operand_value(machine, &src);
                let dst_val = self.get_operand_value(machine, &dst);

                match dst {
                    Operand::Register8(_) | Operand::Mem8(_) => {
                        let dst_val = dst_val as u8;
                        let src_val = src_val as u8;
                        let (result, c) = (dst_val).overflowing_sub(src_val);
                        self.flags.zero = result == 0;
                        self.flags.carry = c;
                        self.flags.sign = (result & 0x80) != 0;
                        self.flags.overflow =
                            ((dst_val ^ src_val) & (dst_val ^ result) & 0x80) != 0;
                        self.flags.parity = result.count_ones() % 2 == 0;
                        self.flags.auxiliary = ((dst_val ^ src_val ^ result) & 0x10) != 0;
                    }
                    Operand::Register16(_) | Operand::Mem16(_) => {
                        let (result, c) = (dst_val).overflowing_sub(src_val);
                        self.flags.zero = result == 0;
                        self.flags.carry = c;
                        self.flags.sign = (result & 0x8000) != 0;
                        self.flags.overflow =
                            ((dst_val ^ src_val) & (dst_val ^ result) & 0x8000) != 0;
                        self.flags.parity = (result as u8).count_ones().is_multiple_of(2);
                        self.flags.auxiliary = ((dst_val ^ src_val ^ result) & 0x10) != 0;
                    }
                    _ => panic!("Invalid combination"),
                }
            }
            Op::Div { src } => {
                let divisor = match src {
                    Operand::Register16(reg) => self.registers.read_u16(reg),
                    Operand::Mem16(spec) => self.read_mem16(machine, &spec),
                    _ => panic!("Div expects r/m16"),
                };

                if divisor == 0 {
                    panic!("Division by zero!");
                }

                let dividend = ((self.registers.read_u16(Register16::Dx) as u32) << 16)
                    | self.registers.read_u16(Register16::Ax) as u32;
                let quotient = (dividend / divisor as u32) as u32;
                let remainder = (dividend % divisor as u32) as u32;

                if quotient > 0xFFFF {
                    panic!("Division overflow");
                }

                self.registers.write_u16(Register16::Ax, quotient as u16);
                self.registers.write_u16(Register16::Dx, remainder as u16);
            }
            Op::IDiv { src } => {
                let divisor = match src {
                    Operand::Register16(reg) => self.registers.read_u16(reg) as i16,
                    Operand::Mem16(spec) => self.read_mem16(machine, &spec) as i16,
                    _ => panic!("Div expects r/m16"),
                };

                if divisor == 0 {
                    panic!("Division by zero!");
                }

                let dividend = (((self.registers.read_u16(Register16::Dx) as u32) << 16)
                    | self.registers.read_u16(Register16::Ax) as u32)
                    as i32;
                let quotient = (dividend / divisor as i32) as i32;
                let reminder = (dividend % divisor as i32) as i32;

                if quotient < i16::MIN as i32 || quotient > i16::MAX as i32 {
                    panic!("Division overflow");
                }

                self.registers.write_u16(Register16::Ax, quotient as u16);
                self.registers.write_u16(Register16::Dx, reminder as u16);
            }
            Op::Mul { src } => match src {
                Operand::Register8(_) | Operand::Mem8(_) => {
                    let lhs = self.registers.read_u8(Register8::Al) as u16;
                    let rhs = self.get_operand_value(machine, &src);

                    let result = lhs * rhs;
                    self.registers.write_u16(Register16::Ax, result);
                    self.flags.carry = result > 0xFF;
                    self.flags.overflow = self.flags.carry;
                }
                Operand::Register16(_) | Operand::Mem16(_) => {
                    let lhs = self.registers.read_u16(Register16::Ax) as u32;
                    let rhs = self.get_operand_value(machine, &src) as u32;

                    let result = lhs * rhs;
                    self.registers.write_u16(Register16::Ax, result as u16);
                    self.registers
                        .write_u16(Register16::Dx, (result >> 16) as u16);
                    self.flags.carry = (result >> 16) != 0;
                    self.flags.overflow = self.flags.carry;
                }
                _ => panic!("Invalid mul operation"),
            },
            Op::IMul { src } => match src {
                Operand::Register8(_) | Operand::Mem8(_) => {
                    let lhs = self.registers.read_u8(Register8::Al) as i8 as i16;
                    let rhs = self.get_operand_value(machine, &src) as u8 as i8 as i16;

                    let result = lhs * rhs;
                    self.registers.write_u16(Register16::Ax, result as u16);
                    self.flags.carry = result < -128 || result > 127;
                    self.flags.overflow = self.flags.carry;
                }
                Operand::Register16(_) | Operand::Mem16(_) => {
                    let lhs = self.registers.read_u16(Register16::Ax) as i16 as i32;
                    let rhs = self.get_operand_value(machine, &src) as i16 as i32;

                    let result = lhs * rhs;
                    self.registers.write_u16(Register16::Ax, result as u16);
                    self.registers
                        .write_u16(Register16::Dx, (result >> 16) as u16);
                    self.flags.carry = result < i16::MIN as i32 || result > i16::MAX as i32;
                    self.flags.overflow = self.flags.carry;
                }
                _ => panic!("Invalid mul operation"),
            },
            Op::Mov { dst, src } => {
                let value = self.get_operand_value(machine, &src);
                self.set_operand_value(machine, &dst, value);
            }
            Op::Int(int) => machine.handle_bios_interrupt(self, int),
            Op::MovSb { rep } => {
                if rep {
                    while self.registers.read_u16(Register16::Cx) != 0 {
                        exec_movsb(self, machine);
                        let cx = self.registers.read_u16(Register16::Cx);
                        self.registers.write_u16(Register16::Cx, cx.wrapping_sub(1));
                    }
                } else {
                    exec_movsb(self, machine);
                }
            }
            Op::MovSw { rep } => {
                if rep {
                    while self.registers.read_u16(Register16::Cx) != 0 {
                        exec_movsw(self, machine);
                        let cx = self.registers.read_u16(Register16::Cx);
                        self.registers.write_u16(Register16::Cx, cx.wrapping_sub(1));
                    }
                } else {
                    exec_movsw(self, machine);
                }
            }
            Op::JmpFar { segment, offset } => {
                self.registers.write_segment(SegmentRegister::Cs, segment);
                self.registers.set_ip(offset);
            }
            Op::Shl { dst, src } => {
                let val = self.get_operand_value(machine, &dst);
                let count = self.get_operand_value(machine, &src);
                if count == 0 {
                    return;
                }
                match dst {
                    Operand::Register8(_) | Operand::Mem8(_) => {
                        let mut v = val as u8;
                        for _ in 0..count {
                            self.flags.carry = (v & 0x80) != 0;
                            v <<= 1;
                        }
                        self.flags.zero = v == 0;
                        self.flags.sign = (v & 0x80) != 0;
                        self.flags.parity = v.count_ones().is_multiple_of(2);
                        if count == 1 {
                            self.flags.overflow = self.flags.carry ^ self.flags.sign;
                        }
                        self.set_operand_value(machine, &dst, v as u16);
                    }
                    Operand::Register16(_) | Operand::Mem16(_) => {
                        let mut v = val;
                        for _ in 0..count {
                            self.flags.carry = (v & 0x8000) != 0;
                            v <<= 1;
                        }
                        self.flags.zero = v == 0;
                        self.flags.sign = (v & 0x8000) != 0;
                        self.flags.parity = (v as u8).count_ones().is_multiple_of(2);
                        if count == 1 {
                            self.flags.overflow = self.flags.carry ^ self.flags.sign;
                        }
                        self.set_operand_value(machine, &dst, v);
                    }
                    _ => panic!("Invalid operand combination"),
                }
            }
            Op::Shr { dst, src } => {
                let val = self.get_operand_value(machine, &dst);
                let count = self.get_operand_value(machine, &src);
                match dst {
                    Operand::Register8(_) | Operand::Mem8(_) => {
                        let mut v = val as u8;
                        let original_msb = (v & 0x80) != 0;
                        for _ in 0..count {
                            self.flags.carry = (v & 0x01) != 0;
                            v >>= 1;
                        }
                        self.flags.zero = v == 0;
                        self.flags.sign = false;
                        self.flags.parity = v.count_ones().is_multiple_of(2);
                        if count == 1 {
                            self.flags.overflow = original_msb;
                        }
                        self.set_operand_value(machine, &dst, v as u16);
                    }
                    Operand::Register16(_) | Operand::Mem16(_) => {
                        let mut v = val;
                        let original_msb = (v & 0x8000) != 0;
                        for _ in 0..count {
                            self.flags.carry = (v & 0x01) != 0;
                            v >>= 1;
                        }
                        self.flags.zero = v == 0;
                        self.flags.sign = false;
                        self.flags.parity = (v as u8).count_ones().is_multiple_of(2);
                        if count == 1 {
                            self.flags.overflow = original_msb;
                        }
                        self.set_operand_value(machine, &dst, v);
                    }
                    _ => panic!("Invalid operand combination"),
                }
            }
            Op::Nop => {}
            Op::Lodsb => {
                let offset = self.registers.read_u16(Register16::Si);
                let segment = self.registers.read_segment(SegmentRegister::Ds);
                let address = Self::calculate_physical_address(segment, offset);
                let value = machine.read_physical_u8(address);
                self.registers.write_u8(Register8::Al, value);
                if self.flags.direction {
                    self.registers
                        .write_u16(Register16::Si, offset.wrapping_sub(1));
                } else {
                    self.registers
                        .write_u16(Register16::Si, offset.wrapping_add(1));
                }
            }
            Op::Out => {
                let port = self.registers.read_u16(Register16::Dx);
                let value = self.registers.read_u8(Register8::Al);
                machine.handle_pic_out(port, value);
            }
            Op::Or { src, dst } => {
                let src_val = self.get_operand_value(machine, &src);
                let dst_val = self.get_operand_value(machine, &dst);

                match dst {
                    Operand::Register8(_) | Operand::Mem8(_) => {
                        let result = (src_val as u8) | (dst_val as u8);

                        self.flags.carry = false;
                        self.flags.overflow = false;
                        self.flags.zero = result == 0;
                        self.flags.sign = result & 0x80 == 0;
                        self.flags.parity = result.count_ones().is_multiple_of(2);

                        self.set_operand_value(machine, &dst, result as u16);
                    }
                    Operand::Register16(_) | Operand::Mem16(_) => {
                        let result = src_val | dst_val;

                        self.flags.carry = false;
                        self.flags.overflow = false;
                        self.flags.zero = result == 0;
                        self.flags.sign = result & 0x8000 == 0;
                        self.flags.parity = (result as u8).count_ones().is_multiple_of(2);

                        self.set_operand_value(machine, &dst, result);
                    }
                    _ => panic!("Invalid combination"),
                }
            }
            _ => panic!("Invalid instruction: {:?}", instruction),
        }
    }

    pub fn resolve_relative(&self, offset: i16) -> u16 {
        self.registers.ip().wrapping_add_signed(offset)
    }

    pub fn read_mem8(&self, machine: &Machine, spec: &MemSpec) -> u8 {
        let offset = self.resolve_address(spec);
        let segment = if let Some(segment) = spec.override_segment {
            segment
        } else if spec.uses_bp() {
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

        let segment = if let Some(segment) = spec.override_segment {
            segment
        } else if spec.uses_bp() {
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

        let segment = if let Some(segment) = spec.override_segment {
            segment
        } else if spec.uses_bp() {
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

        let segment = if let Some(segment) = spec.override_segment {
            segment
        } else if spec.uses_bp() {
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

fn exec_movsw(cpu: &mut Cpu, machine: &mut Machine) {
    let dest_es = cpu.registers.read_segment(SegmentRegister::Es);
    let dest_di = cpu.registers.read_u16(Register16::Di);
    let dst_addr = Cpu::calculate_physical_address(dest_es, dest_di);

    let src_ds = cpu.registers.read_segment(SegmentRegister::Ds);
    let src_si = cpu.registers.read_u16(Register16::Si);
    let src_addr = Cpu::calculate_physical_address(src_ds, src_si);
    let val = machine.read_physical_u16(src_addr);
    machine.write_physical_u16(dst_addr, val);

    if cpu.flags.direction {
        cpu.registers
            .write_u16(Register16::Di, dest_di.wrapping_sub(2));
        cpu.registers
            .write_u16(Register16::Si, src_si.wrapping_sub(2));
    } else {
        cpu.registers
            .write_u16(Register16::Di, dest_di.wrapping_add(2));
        cpu.registers
            .write_u16(Register16::Si, src_si.wrapping_add(2));
    }
}

fn exec_movsb(cpu: &mut Cpu, machine: &mut Machine) {
    let dest_es = cpu.registers.read_segment(SegmentRegister::Es);
    let dest_di = cpu.registers.read_u16(Register16::Di);
    let dst_addr = Cpu::calculate_physical_address(dest_es, dest_di);

    let src_ds = cpu.registers.read_segment(SegmentRegister::Ds);
    let src_si = cpu.registers.read_u16(Register16::Si);
    let src_addr = Cpu::calculate_physical_address(src_ds, src_si);
    let val = machine.read_physical_u8(src_addr);
    machine.write_physical_u8(dst_addr, val);

    if cpu.flags.direction {
        cpu.registers
            .write_u16(Register16::Di, dest_di.wrapping_sub(1));
        cpu.registers
            .write_u16(Register16::Si, src_si.wrapping_sub(1));
    } else {
        cpu.registers
            .write_u16(Register16::Di, dest_di.wrapping_add(1));
        cpu.registers
            .write_u16(Register16::Si, src_si.wrapping_add(1));
    }
}
