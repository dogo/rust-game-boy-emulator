use super::MBC;

pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    bank_reg1: u8,  // bits 0-4 ROM
    bank_reg2: u8,  // bits 5-6 ROM ou RAM
    mode: u8,       // 0=ROM, 1=RAM
}

impl MBC1 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {
        Self {
            rom,
            ram: vec![0; ram_size],
            ram_enabled: false,
            bank_reg1: 1,
            bank_reg2: 0,
            mode: 0,
        }
    }

    fn effective_rom_bank(&self) -> usize {
        let mut bank = self.bank_reg1 as usize;
        if bank == 0 { bank = 1; }
        if self.mode == 0 {
            bank |= ((self.bank_reg2 as usize) & 0x03) << 5;
        }
        bank
    }

    fn effective_ram_bank(&self) -> usize {
        if self.mode == 1 {
            (self.bank_reg2 & 0x03) as usize
        } else {
            0
        }
    }
}

impl MBC for MBC1 {
    fn read_rom(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => {
                let bank = if self.mode == 1 {
                    (self.bank_reg2 as usize & 0x03) << 5
                } else {
                    0
                };
                let idx = bank * 0x4000 + (address as usize);
                self.rom.get(idx).copied().unwrap_or(0xFF)
            }
            0x4000..=0x7FFF => {
                let bank = self.effective_rom_bank();
                let idx = bank * 0x4000 + ((address - 0x4000) as usize);
                self.rom.get(idx).copied().unwrap_or(0xFF)
            }
            _ => 0xFF,
        }
    }

    fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                self.bank_reg1 = value & 0x1F;
            }
            0x4000..=0x5FFF => {
                self.bank_reg2 = value & 0x03;
            }
            0x6000..=0x7FFF => {
                self.mode = value & 0x01;
            }
            _ => {}
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled { return 0xFF; }
        let bank = self.effective_ram_bank();
        let addr = bank * 0x2000 + ((address - 0xA000) as usize);
        self.ram.get(addr).copied().unwrap_or(0xFF)
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled { return; }
        let bank = self.effective_ram_bank();
        let addr = bank * 0x2000 + ((address - 0xA000) as usize);
        if addr < self.ram.len() {
            self.ram[addr] = value;
        }
    }

    fn save_ram(&self) -> Option<Vec<u8>> {
        if self.ram.is_empty() {
            None
        } else {
            Some(self.ram.clone())
        }
    }

    fn load_ram(&mut self, data: &[u8]) {
        let len = data.len().min(self.ram.len());
        self.ram[..len].copy_from_slice(&data[..len]);
    }
}
