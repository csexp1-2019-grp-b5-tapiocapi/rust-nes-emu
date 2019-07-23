pub struct Wram {
    pub ram: Vec<u8>,
}

impl Wram {
    pub fn new(size: usize) -> Wram {
        Wram {
            ram: vec![0; size]
        }
    }

    pub fn reset(&mut self) {
        self.ram = vec![0; self.ram.len()];
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
    pub fn read(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }
}
