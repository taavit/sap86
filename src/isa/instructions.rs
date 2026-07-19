use crate::isa::{Operand, registers::Register16};

#[derive(Debug)]
pub enum Op {
    Nop,
    Dec { dst: Register16 },
    Inc { dst: Register16 },
    Lea { src: Operand, dst: Operand },
    Jnz { addr: Operand },
    Jz { addr: Operand },
    Jmp { addr: Operand },
    Ld { src: Register16 },
    Test { op1: Operand, op2: Operand },
    Mov { src: Operand, dst: Operand },
    Int(u8),
    Halt,
}
