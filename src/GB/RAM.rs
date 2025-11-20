pub struct RAM {
    // all the 65,536 addressable locations
    memory: [u8; 65536],
}

impl RAM {
    pub fn new() -> Self {
        RAM { memory: [0; 65536] }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte;
    }

    pub fn load_bytes(&mut self, data: &[u8]) {
        let len = data.len().min(self.memory.len());
        self.memory[..len].copy_from_slice(&data[..len]);
    }
}