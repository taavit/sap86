#[derive(Debug)]
pub struct Flags {
    pub zero: bool,
    pub interrupt: bool,
    pub direction: bool,
    pub carry: bool,
}
