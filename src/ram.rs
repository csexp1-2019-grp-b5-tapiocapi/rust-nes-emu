pub struct Ram {
    pub ram: Vec<u8>,
}

impl Ram {
    pub fn new(size: usize) -> Self {
        Self { ram: vec![0; size] }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
    pub fn read(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }
}
