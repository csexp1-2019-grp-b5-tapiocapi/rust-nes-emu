pub struct Rom {
    pub rom: Vec<u8>,
}

impl Rom {
    pub fn init(entire_rom: &Vec<u8>, range: (u16, u16)) -> Rom {
        let from = range.0 as usize;
        let to = range.1 as usize;

        //println!("{:?}", entire_rom[from..=to].to_vec()/*.len()*/);
        Rom {
            rom: entire_rom[from..=to].to_vec(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        return self.rom[addr as usize];
    }
}
