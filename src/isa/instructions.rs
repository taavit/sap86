use crate::isa::Operand;

#[derive(Debug)]
pub enum Op {
    Nop,
    Cli,
    Sti,
    Lea { src: Operand, dst: Operand },
    Jnz { addr: Operand },
    Jz { addr: Operand },
    Jmp { addr: Operand },
    Inc { dst: Operand },
    Test { op1: Operand, op2: Operand },
    Mov { src: Operand, dst: Operand },
    Sub { src: Operand, dst: Operand },
    Int(u8),
    Halt,
}
