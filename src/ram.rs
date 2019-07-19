pub struct Ram {
    pub ram: Vec<u8>,
}

impl Ram {
    pub fn init(size: usize) -> Ram {
        return Ram { ram: vec![0; size] };
    }

    pub fn reset(&self) {}
    pub fn write(&self) {}
    pub fn read(&self, addr: u16) {}
}
