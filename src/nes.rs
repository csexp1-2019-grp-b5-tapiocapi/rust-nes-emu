use crate::cpu::Cpu;
use crate::cpu_bus::CpuBus;
use crate::ram::Ram;
use crate::rom::Rom;

use std::io;
use std::path::Path;

pub struct Nes {
    game_rom: Vec<u8>,
    prog_range: (u16, u16), //GAME ROM
    chr_range: (u16, u16),  //CHR  ROM
}

impl Nes {
    pub fn load<P: AsRef<Path>>(file_path: P) -> io::Result<Nes> {
        let buffer = std::fs::read(file_path.as_ref())?;
        //println!("len:{:x}", buffer.len());
        let header_size = 0x0010;

        let program_rom_size = buffer[4] as u16;
        let character_rom_size = buffer[5] as u16;
        let trainer_size = if (buffer[6] & 0b0000_0010) == 0b0000_0010 {
            0x200
        } else {
            0x0
        };

        let character_rom_start = header_size + trainer_size + program_rom_size * 0x4000; //16KiB -> 0x4000
        let character_rom_end = character_rom_start + character_rom_size * 0x2000; //8Kib  -> 0x2000
        let game = Nes {
            game_rom: buffer,
            prog_range: (header_size, character_rom_start - 1),
            chr_range: (character_rom_start, character_rom_end - 1),
        };

        Ok(game)
    }

    pub fn start(&self) {
        let rom = Rom::init(&self.game_rom, self.prog_range);
        let ram = Ram::init(2048);

        //cpu.run();
        let bus = CpuBus::init(ram, rom);
        let mut cpu = Cpu::init(bus);
        //pirintln!("{:?}", self.game_rom);
        cpu.reset();
        loop {
            cpu.run();
        }
    }
}
