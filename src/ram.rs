pub struct Ram {
    pub ram: Vec<u8>
}

pub fn init(size: usize) -> Ram {
    return Ram {
        ram: vec![0; size]
    };
}

impl Ram {
    pub fn reset(&self) {
    }
    pub fn write(&self) {
    }
    pub fn read(&self, addr: u16) {
    }
}
