#[derive(Debug, Default)]
pub struct Flags {
    pub zero: bool,
    pub interrupt: bool,
    pub direction: bool,
    pub carry: bool,
    pub overflow: bool,
    pub sign: bool,
    pub parity: bool,
    pub auxiliary: bool,
}
