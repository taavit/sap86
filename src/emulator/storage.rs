pub struct GenericFloppy<
    const CYLINDERS: usize,
    const HEADS: usize,
    const SECTORS_PER_TRACK: usize,
    const SECTOR_SIZE: usize,
> {
    data: Vec<u8>,
}

impl<
    const CYLINDERS: usize,
    const HEADS: usize,
    const SECTORS_PER_TRACK: usize,
    const SECTOR_SIZE: usize,
> GenericFloppy<CYLINDERS, HEADS, SECTORS_PER_TRACK, SECTOR_SIZE>
{
    pub const CAPACITY: usize = CYLINDERS * HEADS * SECTORS_PER_TRACK * SECTOR_SIZE;

    fn chs_to_lba(&self, c: u8, h: u8, s: u8) -> usize {
        let track_offset = (c as usize * HEADS + h as usize) * SECTORS_PER_TRACK;
        let sector_offset = s as usize - 1;
        (track_offset + sector_offset) * SECTOR_SIZE
    }
    pub fn from_image(image: &[u8]) -> Self {
        Self {
            data: image.to_vec(),
        }
    }
}

impl<
    const CYLINDERS: usize,
    const HEADS: usize,
    const SECTORS_PER_TRACK: usize,
    const SECTOR_SIZE: usize,
> Floppy for GenericFloppy<CYLINDERS, HEADS, SECTORS_PER_TRACK, SECTOR_SIZE>
{
    fn read_chs_sectors(&self, c: u8, h: u8, s: u8, count: u8) -> &[u8] {
        let start_lba = self.chs_to_lba(c, h, s);
        let start_byte = start_lba;
        let bytes_to_read = count as usize * SECTOR_SIZE;
        &self.data[start_byte..start_byte + bytes_to_read]
    }
}

pub trait Floppy {
    fn read_chs_sectors(&self, c: u8, h: u8, s: u8, count: u8) -> &[u8];
}

pub type Floppy525_160 = GenericFloppy<40, 1, 8, 512>;
pub type Floppy525_360 = GenericFloppy<40, 2, 9, 512>;
