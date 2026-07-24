use crate::{
    emulator::{cpu::Cpu, machine::Machine},
    isa::Operand,
};

pub fn exec_sub(src: &Operand, dst: &Operand, cpu: &mut Cpu, machine: &mut Machine) {
    let src_val = cpu.get_operand_value(machine, src);
    let dst_val = cpu.get_operand_value(machine, dst);

    let result = match dst {
        Operand::Register8(_) | Operand::Mem8(_) => {
            let dst_val = dst_val as u8;
            let src_val = src_val as u8;
            let (result, c) = (dst_val).overflowing_sub(src_val);

            cpu.flags.carry = c;
            cpu.flags.sign = (result & 0x80) != 0;
            cpu.flags.overflow = ((dst_val ^ src_val) & (dst_val ^ result) & 0x80) != 0;
            cpu.flags.auxiliary = ((dst_val ^ src_val ^ result) & 0x10) != 0;
            cpu.set_operand_value_8(machine, dst, result).unwrap();
            result as u16
        }
        Operand::Register16(_) | Operand::Mem16(_) => {
            let (result, c) = (dst_val).overflowing_sub(src_val);
            cpu.flags.carry = c;
            cpu.flags.sign = (result & 0x8000) != 0;
            cpu.flags.overflow = ((dst_val ^ src_val) & (dst_val ^ result) & 0x8000) != 0;

            cpu.flags.auxiliary = ((dst_val ^ src_val ^ result) & 0x10) != 0;
            cpu.set_operand_value_16(machine, dst, result).unwrap();
            result
        }
        _ => panic!("Invalid combination"),
    };

    cpu.flags.zero = result == 0;
    cpu.flags.parity = (result as u8).count_ones().is_multiple_of(2);
}
