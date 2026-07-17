use crate::compiler::ModRm;

pub mod flags;
pub mod registers;

#[derive(Debug)]
pub enum MemorySpec {
    MemoryBx,
    Displacement(u16),
}
