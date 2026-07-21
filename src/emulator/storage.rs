const SECTOR_SIZE: usize = 512;
const HEADS: usize = 2;
const SECTORS_PER_TRACK: usize = 18;
const CYLINDERS: usize = 80;
const FLOPPY_SIZE_525DD: usize = CYLINDERS * HEADS * SECTORS_PER_TRACK * SECTOR_SIZE;

// const FLOPPY_SIZE_525DD: usize = CYLINDERS * HEADS * SECTORS_PER_TRACK * SECTOR_SIZE;

pub struct Floppy525DD {
    data: [u8; FLOPPY_SIZE_525DD],
}

pub trait Floppy {
    fn read_chs_sector(&self, c: u8, h: u8, s: u8) -> &[u8];
    fn reset_device(&mut self);
}

impl Floppy for Floppy525DD {
    fn read_chs_sector(&self, c: u8, h: u8, s: u8) -> &[u8] {
        let lba = Self::chs_to_lba(c, h, s) as usize;
        let pos = lba * SECTOR_SIZE;
        &self.data[pos..pos + SECTOR_SIZE]
    }
    fn reset_device(&mut self) {}
}

impl Floppy525DD {
    pub fn new() -> Self {
        Self {
            data: [0; FLOPPY_SIZE_525DD],
        }
    }
    fn chs_to_lba(c: u8, h: u8, s: u8) -> u32 {
        (c as u32 * 2 + h as u32) * 9 + (s as u32 - 1)
    }

    pub fn insert(&mut self, data: &[u8]) {
        self.data[..data.len()].copy_from_slice(data);
    }
}
