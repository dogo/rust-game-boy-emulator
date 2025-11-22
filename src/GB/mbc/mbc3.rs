use super::MBC;

pub struct MBC3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,
    rtc: [u8; 5], // s, m, h, dl, dh
    rtc_latch: [u8; 5],
    rtc_latch_state: u8,
}

impl MBC3 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {
        Self {
            rom,
            ram: vec![0; ram_size],
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            rtc: [0; 5],
            rtc_latch: [0; 5],
            rtc_latch_state: 0,
        }
    }
}

impl MBC for MBC3 {
    fn read_rom(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom.get(address as usize).copied().unwrap_or(0xFF),
            0x4000..=0x7FFF => {
                // Banco 0 nunca pode ser selecionado em 0x4000–0x7FFF (hardware substitui por banco 1)
                let bank = (self.rom_bank as usize).max(1);
                let idx = bank * 0x4000 + ((address - 0x4000) as usize);
                self.rom.get(idx).copied().unwrap_or(0xFF)
            }
            _ => 0xFF,
        }
    }
    fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = (value & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                let mut bank = value & 0x7F;
                if bank == 0 { bank = 1; }
                self.rom_bank = bank;
            }
            0x4000..=0x5FFF => self.ram_bank = value,
            0x6000..=0x7FFF => {
                if self.rtc_latch_state == 0x00 && value == 0x01 {
                    self.rtc_latch.copy_from_slice(&self.rtc);
                }
                self.rtc_latch_state = value;
            }
            _ => {}
        }
    }
    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled { return 0xFF; }
        match self.ram_bank {
            0x00..=0x03 => {
                let idx = (self.ram_bank as usize) * 0x2000 + ((address - 0xA000) as usize);
                self.ram.get(idx).copied().unwrap_or(0xFF)
            }
            0x08..=0x0C => self.rtc_latch[(self.ram_bank - 0x08) as usize],
            _ => 0xFF,
        }
    }
    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled { return; }
        match self.ram_bank {
            0x00..=0x03 => {
                let idx = (self.ram_bank as usize) * 0x2000 + ((address - 0xA000) as usize);
                if idx < self.ram.len() {
                    self.ram[idx] = value;
                }
            }
            0x08..=0x0C => {
                let reg = (self.ram_bank - 0x08) as usize;
                self.rtc[reg] = value;
            }
            _ => {}
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
