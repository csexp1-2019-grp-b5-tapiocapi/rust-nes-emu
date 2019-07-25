use crate::cpu::Cpu;
use crate::cpu_bus::CpuBus;
use crate::ppu;
use crate::ram::Ram;
use crate::rom;

use std::io;
use std::path::Path;

pub const CV_WINDOW_TITLE: &str = "Tapioca-NES";

pub struct Nes {
    cpu: Cpu,
}

impl Nes {
    pub fn load<P: AsRef<Path>>(file_path: P) -> io::Result<Nes> {
        let buffer = std::fs::read(file_path.as_ref())?;

        let (prog, chr) = rom::load(buffer);
        let wram = Ram::new(2048);
        let ppu = ppu::Ppu::new(&chr);

        let cpu_bus = CpuBus::new(wram, prog, chr, ppu);

        Ok(Nes {
            cpu: Cpu::new(cpu_bus),
        })
    }

    pub fn start(&mut self) {
        //pirintln!("{:?}", self.game_rom);
        self.cpu.reset();
        loop {
            self.cpu.run();
        }
    }
}
