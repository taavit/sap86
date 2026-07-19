#[derive(Debug)]
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
