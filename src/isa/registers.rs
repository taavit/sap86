use std::fmt::{Debug, Display};

pub struct Registers {
    gpr: [u16; 8],
    sreg: [u16; 4],
    ip: u16,
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

impl Registers {
    pub fn new() -> Self {
        Self {
            gpr: [0; 8],
            sreg: [0; 4],
            ip: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentRegister {
    Es = 0,
    Cs = 1,
    Ss = 2,
    Ds = 3,
}

impl Display for SegmentRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cs => f.write_str("cs"),
            Self::Ds => f.write_str("ds"),
            Self::Es => f.write_str("es"),
            Self::Ss => f.write_str("ss"),
        }
    }
}

impl From<u8> for SegmentRegister {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => SegmentRegister::Es,
            0b01 => SegmentRegister::Cs,
            0b10 => SegmentRegister::Ss,
            0b11 => SegmentRegister::Ds,
            _ => unreachable!(),
        }
    }
}

impl Registers {
    pub fn step_ip(&mut self) {
        self.ip = self.ip.wrapping_add(1);
    }

    pub fn step_ip_by(&mut self, step: u16) {
        self.ip = self.ip.wrapping_add(step);
    }

    pub fn ip(&self) -> u16 {
        self.ip
    }

    pub fn set_ip(&mut self, ip: u16) {
        self.ip = ip;
    }

    pub fn write_u16(&mut self, reg: Register16, value: u16) {
        self.gpr[reg as usize] = value;
    }
    pub fn read_u16(&self, reg: Register16) -> u16 {
        self.gpr[reg as usize]
    }

    pub fn write_u8(&mut self, reg: Register8, value: u8) {
        if (reg as usize) < 4 {
            self.gpr[reg as usize] = (self.gpr[reg as usize] & 0xFF00) | value as u16
        } else {
            self.gpr[reg as usize - 4] =
                (self.gpr[reg as usize - 4] & 0x00FF) | ((value as u16) << 8)
        }
    }

    pub fn read_u8(&self, reg: Register8) -> u8 {
        if (reg as usize) < 4 {
            (self.gpr[reg as usize] & 0xFF) as u8
        } else {
            ((self.gpr[reg as usize - 4] >> 8) & 0xFF) as u8
        }
    }

    pub fn read_segment(&self, reg: SegmentRegister) -> u16 {
        self.sreg[reg as usize]
    }

    pub fn write_segment(&mut self, reg: SegmentRegister, v: u16) {
        self.sreg[reg as usize] = v;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Register16 {
    Ax,
    Cx,
    Dx,
    Bx,
    Sp,
    Bp,
    Si,
    Di,
}

impl Display for Register16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register16::Ax => f.write_str("ax"),
            Register16::Cx => f.write_str("cx"),
            Register16::Dx => f.write_str("dx"),
            Register16::Bx => f.write_str("bx"),
            Register16::Sp => f.write_str("sp"),
            Register16::Bp => f.write_str("bp"),
            Register16::Si => f.write_str("si"),
            Register16::Di => f.write_str("di"),
        }
    }
}

impl Debug for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "[REGS ] AX: 0x{:04X}\tCX: 0x{:04X}\tDX: 0x{:04X}\tBX: 0x{:04X}\tSP: 0x{:04X}\tBP: 0x{:04X}\tSI: 0x{:04X}\tDI: 0x{:04X}",
            self.gpr[0],
            self.gpr[1],
            self.gpr[2],
            self.gpr[3],
            self.gpr[4],
            self.gpr[5],
            self.gpr[6],
            self.gpr[7],
        )?;
        writeln!(
            f,
            "[REGS ] IP: 0x{:04X}\tES: 0x{:04X}\tCS: 0x{:04X}\tSS: 0x{:04X}\tDS: 0x{:04X}",
            self.ip, self.sreg[0], self.sreg[1], self.sreg[2], self.sreg[3],
        )?;
        Ok(())
    }
}

impl From<u8> for Register16 {
    fn from(value: u8) -> Self {
        match value {
            0 => Register16::Ax,
            1 => Register16::Cx,
            2 => Register16::Dx,
            3 => Register16::Bx,
            4 => Register16::Sp,
            5 => Register16::Bp,
            6 => Register16::Si,
            7 => Register16::Di,
            _ => panic!("Invalid value"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Register8 {
    Al,
    Cl,
    Dl,
    Bl,
    Ah,
    Ch,
    Dh,
    Bh,
}

impl Display for Register8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Al => f.write_str("al"),
            Self::Ah => f.write_str("ah"),
            Self::Cl => f.write_str("cl"),
            Self::Ch => f.write_str("ch"),
            Self::Dl => f.write_str("dl"),
            Self::Dh => f.write_str("dh"),
            Self::Bl => f.write_str("bl"),
            Self::Bh => f.write_str("bh"),
        }
    }
}

impl From<u8> for Register8 {
    fn from(value: u8) -> Self {
        match value {
            0 => Register8::Al,
            1 => Register8::Cl,
            2 => Register8::Dl,
            3 => Register8::Bl,
            4 => Register8::Ah,
            5 => Register8::Ch,
            6 => Register8::Dh,
            7 => Register8::Bh,
            _ => panic!("Invalid value"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::isa::registers::{Register16, Registers};

    #[test]
    fn test_registers() {
        let mut registers = Registers::new();
        registers.write_u8(super::Register8::Al, 0x34);
        registers.write_u8(super::Register8::Ah, 0x12);
        assert_eq!(registers.read_u16(Register16::Ax), 0x1234);
    }
}
