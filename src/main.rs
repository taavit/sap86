use std::{fs::File, io::Read};

use crate::emulator::{cpu::Cpu, machine::Machine, memory::Memory};

mod emulator;
mod isa;

fn main() {
    let mut cpu = Cpu::new();
    let memory = Memory::new();
    let mut machine = Machine { memory };
    let path: Vec<String> = std::env::args().collect();
    let mut file = File::open(&path[1]).unwrap();
    let mut buf = Vec::new();
    let program_size = file.read_to_end(&mut buf).unwrap();
    eprintln!("[COMPILER] Size = {}", program_size);
    machine.load_program(&buf);
    loop {
        let instruction = cpu.fetch_decode(&mut machine);
        cpu.execute(&mut machine, instruction);
        if cpu.halted {
            break;
        }
    }
}
