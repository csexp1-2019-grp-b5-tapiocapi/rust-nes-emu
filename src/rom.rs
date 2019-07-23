pub const INES_HEADER_SIZE: usize = 0x0010;

pub fn load_rom(rom: Vec<u8>) -> (ProgramRom, CharacterRom) {
    let program_rom_size = rom[4] as usize;
    let character_rom_size = rom[5] as usize;

    let trainer_size = if (rom[6] & 0b0000_0010) == 0b0000_0010 {
        0x200
    } else {
        0x0
    };

    let character_rom_start = (INES_HEADER_SIZE + trainer_size + program_rom_size * 0x4000) as usize;  //16KiB -> 0x4000
    let character_rom_end = (character_rom_start + character_rom_size * 0x2000) as usize;  //8Kib  -> 0x2000

    (
        ProgramRom::new(&rom[INES_HEADER_SIZE..character_rom_start]),
        CharacterRom::new(&rom[character_rom_start..character_rom_end])
    )
}

pub struct ProgramRom {
    data: Vec<u8>,
}

impl ProgramRom {
    pub fn new(data: &[u8]) -> ProgramRom {
        ProgramRom {
            data: data.to_vec(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        return self.data[addr as usize];
    }
}


pub struct CharacterRom {
    data: Vec<u8>,
}

impl CharacterRom {
    pub fn new(data: &[u8]) -> CharacterRom {
        CharacterRom {
            data: data.to_vec(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        return self.data[addr as usize];
    }
}
