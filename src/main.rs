use std::{fs::File, io::Read};

use crate::{
    emulator::{
        cpu::Cpu,
        decoder::fetch_decode,
        machine::{Machine, VideoCard},
        memory::Memory,
        storage::Floppy525DD,
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
        floppy: Floppy525DD::new(),
    };
    let path: Vec<String> = std::env::args().collect();
    let mut file = File::open(&path[1]).unwrap();
    let mut buf = Vec::new();
    let program_size = file.read_to_end(&mut buf).unwrap();
    eprintln!("[EMU ] Size = {}", program_size);
    machine.boot(&mut cpu, &buf);
    loop {
        let segment = cpu.registers.read_segment(SegmentRegister::Cs);
        let offset = cpu.registers.ip();
        let instruction = fetch_decode(&mut cpu, &mut machine);
        eprintln!("[{segment:04X}:{offset:04X}] {}", instruction);
        cpu.execute(&mut machine, instruction);
        if cpu.halted {
            break;
        }
    }
}
