pub struct Rom {
    pub rom: Vec<u8>,
}

pub fn init(entire_rom: Vec<u8>, range: (u16, u16)) -> Rom {
    let size = (range.1 - range.0) as usize;
    let mut game_rom: Vec<u8> = vec![0; size];
    for index in range.0 as usize .. range.1 as usize {
        game_rom.push(entire_rom[index]);
    }

    let rom = Rom {rom: game_rom};

    return rom;
}

impl Rom {
    //pub fn reset(&self) {
    //}
    //pub fn write(&self) {
    //}
    pub fn read(self, addr: u16) -> u8 {
        return self.rom[addr as usize];
    }
}
