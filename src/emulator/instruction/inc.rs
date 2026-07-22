use crate::{
    emulator::{cpu::Cpu, instruction::exec_add, machine::Machine},
    isa::Operand,
};

pub fn exec_inc(dst: &Operand, cpu: &mut Cpu, machine: &mut Machine) {
    let current_carry_flag = cpu.flags.carry;

    match dst {
        Operand::Register8(_) | Operand::Mem8(_) => {
            exec_add(&Operand::Imm8(1), dst, cpu, machine);
        }
        Operand::Register16(_) | Operand::Mem16(_) => {
            exec_add(&Operand::Imm16(1), dst, cpu, machine);
        }
        _ => panic!("Invalid combination"),
    };
    cpu.flags.carry = current_carry_flag;
}
