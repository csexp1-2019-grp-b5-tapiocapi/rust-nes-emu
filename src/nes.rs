use std::io::Read;
use std::fs::File;

use crate::cpu;
use crate::cpu_bus;
use crate::ram;
use crate::rom;

pub struct Nes {
    pub game_rom: Vec<u8>,
    pub prog_range: (u16, u16), //GAME ROM
    pub chr_range: (u16, u16)   //CHR  ROM
}

pub fn load(file_path: &str) -> Nes {
    let mut rom_file = File::open(file_path).unwrap();
    let mut buffer = Vec::new();
    let header_size  = 0x0010;

    let _ = rom_file.read_to_end(&mut buffer).unwrap();
    let program_rom_size = buffer[4] as u16;
    let character_rom_size = buffer[5] as u16;

    let character_rom_start = header_size + program_rom_size * 0x4000;  //16KiB -> 0x4000
    let character_rom_end = character_rom_start + character_rom_size * 0x2000; //8Kib  -> 0x2000
    let game = Nes {
        game_rom: buffer,
        prog_range: (header_size, character_rom_start - 1),
        chr_range: (character_rom_start, character_rom_end - 1)
    };

    return game;

}

impl Nes {
    pub fn start(self) {
        let rom = rom::init(self.game_rom, self.prog_range);
        let mut ram = ram::init(2048);

        //cpu.run();
        let bus = cpu_bus::init(ram, rom);
        let mut cpu = cpu::init(bus);
    }
}

