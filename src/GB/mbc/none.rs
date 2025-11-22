use super::MBC;

pub struct NoMBC {
    rom: Vec<u8>,
}

impl NoMBC {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { rom }
    }
}

impl MBC for NoMBC {
    fn read_rom(&self, address: u16) -> u8 {
        self.rom.get(address as usize).copied().unwrap_or(0xFF)
    }
    fn write_register(&mut self, _address: u16, _value: u8) {}
    fn read_ram(&self, _address: u16) -> u8 {
        0xFF
    }
    fn write_ram(&mut self, _address: u16, _value: u8) {}
    fn save_ram(&self) -> Option<Vec<u8>> {
        None
    }
    fn load_ram(&mut self, _data: &[u8]) {}
}
