use super::MBC;

/// ROM sem controlador, opcionalmente com até 8 KB de RAM em $A000-$BFFF (tipos $08/$09)
pub struct NoMBC {
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl NoMBC {
    pub fn new(rom: Vec<u8>) -> Self {
        Self::with_ram(rom, 0)
    }

    pub fn with_ram(rom: Vec<u8>, ram_size: usize) -> Self {
        Self {
            rom,
            ram: vec![0; ram_size.min(0x2000)],
        }
    }
}

impl MBC for NoMBC {
    fn read_rom(&self, address: u16) -> u8 {
        self.rom.get(address as usize).copied().unwrap_or(0xFF)
    }
    fn write_register(&mut self, _address: u16, _value: u8) {}
    fn read_ram(&self, address: u16) -> u8 {
        let offset = address.wrapping_sub(0xA000) as usize;
        self.ram.get(offset).copied().unwrap_or(0xFF)
    }
    fn write_ram(&mut self, address: u16, value: u8) {
        let offset = address.wrapping_sub(0xA000) as usize;
        if let Some(byte) = self.ram.get_mut(offset) {
            *byte = value;
        }
    }
    fn save_ram(&self) -> Option<Vec<u8>> {
        super::battery_save(&self.rom, &self.ram)
    }
    fn load_ram(&mut self, data: &[u8]) -> Result<(), String> {
        super::load_exact(&self.rom, &mut self.ram, data)
    }
}
