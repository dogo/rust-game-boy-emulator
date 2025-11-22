use super::MBC;

pub struct MBC2 {
    rom: Vec<u8>,
    ram: [u8; 512],
    ram_enabled: bool,
    rom_bank: u8,
}

impl MBC2 {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom,
            ram: [0; 512],
            ram_enabled: false,
            rom_bank: 1,
        }
    }
}

impl MBC for MBC2 {
    fn read_rom(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom.get(address as usize).copied().unwrap_or(0xFF),
            0x4000..=0x7FFF => {
                let idx = (self.rom_bank as usize) * 0x4000 + ((address - 0x4000) as usize);
                self.rom.get(idx).copied().unwrap_or(0xFF)
            }
            _ => 0xFF,
        }
    }
    fn write_register(&mut self, address: u16, value: u8) {
        if address & 0x0100 == 0 {
            self.ram_enabled = (value & 0x0F) == 0x0A;
        } else {
            self.rom_bank = (value & 0x0F).max(1);
        }
    }
    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled || !(0xA000..=0xA1FF).contains(&address) {
            return 0xFF;
        }
        let idx = (address - 0xA000) as usize & 0x1FF;
        self.ram[idx] | 0xF0
    }
    fn write_ram(&mut self, address: u16, value: u8) {
        if self.ram_enabled && (0xA000..=0xA1FF).contains(&address) {
            let idx = (address - 0xA000) as usize & 0x1FF;
            self.ram[idx] = value & 0x0F;
        }
    }
    fn save_ram(&self) -> Option<Vec<u8>> {
        Some(self.ram.to_vec())
    }
    fn load_ram(&mut self, data: &[u8]) {
        let len = data.len().min(512);
        self.ram[..len].copy_from_slice(&data[..len]);
    }
}
