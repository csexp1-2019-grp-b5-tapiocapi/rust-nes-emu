use crate::cpu::Cpu;
use crate::cpu_bus::CpuBus;
use crate::ram::Ram;
use crate::rom;

use std::io;
use std::path::Path;

pub struct Nes {
    cpu: Cpu,
}

impl Nes {
    pub fn load<P: AsRef<Path>>(file_path: P) -> io::Result<Nes> {
        let buffer = std::fs::read(file_path.as_ref())?;

        let (prog, chr) = rom::load(buffer);
        let wram = Ram::new(2048);

        let cpu_bus = CpuBus::new(wram, prog, chr);

        Ok(Nes {
            cpu: Cpu::new(cpu_bus),
        })
    }

    pub fn start(&mut self) {
        //pirintln!("{:?}", self.game_rom);
        self.cpu.reset();
        let mut a: u64 = 0;
        loop {
            if a < 100 {
                self.cpu.run();
            }
            a += 1;
        }
    }
}
