use super::MBC;

pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    bank_reg1: u8, // bits 0-4 ROM
    bank_reg2: u8, // bits 5-6 ROM ou RAM
    mode: u8,      // 0=ROM, 1=RAM
    multicart: bool,
}

impl MBC1 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {
        let multicart = Self::detect_multicart(&rom);
        Self {
            rom,
            ram: vec![0; ram_size],
            ram_enabled: false,
            bank_reg1: 1,
            bank_reg2: 0,
            mode: 0,
            multicart,
        }
    }

    fn detect_multicart(rom: &[u8]) -> bool {
        const NINTENDO_LOGO: [u8; 48] = [
            0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C,
            0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6,
            0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC,
            0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
        ];

        if rom.len() != 64 * 0x4000 {
            return false;
        }

        [16, 32, 48].iter().all(|bank| {
            let start = bank * 0x4000 + 0x0104;
            rom.get(start..start + NINTENDO_LOGO.len()) == Some(&NINTENDO_LOGO)
        })
    }

    fn rom_bank_count(&self) -> usize {
        (self.rom.len() / 0x4000).max(1)
    }

    fn ram_bank_count(&self) -> usize {
        (self.ram.len() / 0x2000).max(1)
    }

    fn rom_bank2_shift(&self) -> usize {
        if self.multicart { 4 } else { 5 }
    }

    fn effective_upper_rom_bank(&self) -> usize {
        let high = ((self.bank_reg2 as usize) & 0x03) << self.rom_bank2_shift();
        let raw_low = self.bank_reg1 & 0x1F;
        let low = match raw_low {
            0 => 1,
            value if self.multicart => (value & 0x0F) as usize,
            value => value as usize,
        };

        (high | low) % self.rom_bank_count()
    }

    fn effective_lower_rom_bank(&self) -> usize {
        let bank = if self.mode == 1 {
            ((self.bank_reg2 as usize) & 0x03) << self.rom_bank2_shift()
        } else {
            0
        };

        bank % self.rom_bank_count()
    }

    fn effective_ram_bank(&self) -> usize {
        if self.ram.is_empty() || self.mode == 0 {
            0
        } else {
            (self.bank_reg2 as usize & 0x03) % self.ram_bank_count()
        }
    }
}

impl MBC for MBC1 {
    fn read_rom(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => {
                let bank = self.effective_lower_rom_bank();
                let idx = bank * 0x4000 + (address as usize);
                self.rom.get(idx).copied().unwrap_or(0xFF)
            }
            0x4000..=0x7FFF => {
                let bank = self.effective_upper_rom_bank();
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
        if !self.ram_enabled {
            return 0xFF;
        }
        let bank = self.effective_ram_bank();
        let addr = bank * 0x2000 + ((address - 0xA000) as usize);
        self.ram.get(addr).copied().unwrap_or(0xFF)
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }
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
