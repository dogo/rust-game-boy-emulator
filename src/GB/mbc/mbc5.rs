use super::MBC;

pub struct MBC5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    rom_bank: u16,
    ram_bank: u8,
}

impl MBC5 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {
        Self {
            rom,
            ram: vec![0; ram_size],
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
        }
    }
}

impl MBC for MBC5 {
    fn read_rom(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom.get(address as usize).copied().unwrap_or(0xFF),
            0x4000..=0x7FFF => {
                // MBC5 permite banco 0 em 0x4000–0x7FFF (diferente do MBC3)
                let bank = self.rom_bank as usize;
                let idx = bank * 0x4000 + ((address - 0x4000) as usize);
                self.rom.get(idx).copied().unwrap_or(0xFF)
            }
            _ => 0xFF,
        }
    }

    fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = (value & 0x0F) == 0x0A,
            0x2000..=0x2FFF => self.rom_bank = (self.rom_bank & 0x100) | value as u16,
            0x3000..=0x3FFF => self.rom_bank = (self.rom_bank & 0xFF) | (((value & 0x01) as u16) << 8),
            0x4000..=0x5FFF => self.ram_bank = value & 0x0F,
            _ => {}
        }
    }
    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled { return 0xFF; }
        let bank = (self.ram_bank & 0x0F) as usize;
        let idx = bank * 0x2000 + ((address - 0xA000) as usize);
        self.ram.get(idx).copied().unwrap_or(0xFF)
    }
    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled { return; }
        let bank = (self.ram_bank & 0x0F) as usize;
        let idx = bank * 0x2000 + ((address - 0xA000) as usize);
        if idx < self.ram.len() {
            self.ram[idx] = value;
        }
    }
    fn save_ram(&self) -> Option<Vec<u8>> {
        if self.ram.is_empty() { None } else { Some(self.ram.clone()) }
    }
    fn load_ram(&mut self, data: &[u8]) {
        let len = data.len().min(self.ram.len());
        self.ram[..len].copy_from_slice(&data[..len]);
    }
}
