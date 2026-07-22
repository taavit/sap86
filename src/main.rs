use std::{
    fs::File,
    io::{self, Read},
};

use crate::{
    emulator::{
        cpu::Cpu,
        decoder::fetch_decode,
        machine::{Machine, VideoCard},
        memory::Memory,
        storage::{Floppy, Floppy35_144, Floppy525_160, Floppy525_360},
    },
    isa::registers::SegmentRegister,
};

mod emulator;
mod isa;

fn main() {
    let mut cpu = Cpu::new();
    let memory = Memory::new();
    let mut machine = Machine {
        memory,
        video: VideoCard::default(),
        floppy: None,
    };
    let path: Vec<String> = std::env::args().collect();
    let mut file = File::open(&path[1]).unwrap();
    let mut buf = Vec::new();
    let program_size = file.read_to_end(&mut buf).unwrap();
    let floppy: Box<dyn Floppy> = match program_size {
        Floppy525_360::CAPACITY => Box::new(Floppy525_360::from_image(&buf)),
        Floppy525_160::CAPACITY => Box::new(Floppy525_160::from_image(&buf)),
        Floppy35_144::CAPACITY => Box::new(Floppy35_144::from_image(&buf)),
        _ => panic!("Unknown floppy"),
    };
    eprintln!("[EMU ] Size = {}", program_size);
    machine.insert_floppy(floppy);
    machine.boot(&mut cpu);
    let mut i = 0;
    let mut s = String::new();
    loop {
        let segment = cpu.registers.read_segment(SegmentRegister::Cs);
        let offset = cpu.registers.ip();
        let instruction = fetch_decode(&mut cpu, &mut machine);
        eprintln!("[{segment:04X}:{offset:04X}] {}", instruction);
        cpu.execute(&mut machine, instruction);
        if cpu.halted {
            break;
        }
        if i % 100 == 0 {
            io::stdin().read_line(&mut s).unwrap();
        }
        i += 1;
    }
}
