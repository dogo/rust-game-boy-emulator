use crate::GB::APU;
use crate::GB::PPU;
use crate::GB::joypad::Joypad;
use crate::GB::mbc::MBC;
use crate::GB::timer::Timer;
use rand::Rng;

pub struct MemoryBus {
    mbc: Box<dyn MBC>,
    wram: [u8; 0x2000], // Work RAM (8KB)
    hram: [u8; 0x7F],   // High RAM (127 bytes)
    timer: Timer,
    pub joypad: Joypad,
    pub ppu: PPU::PPU,
    pub apu: APU::APU,
    tima: u8,                  // FF05
    tma: u8,                   // FF06
    tac: u8,                   // FF07
    ie: u8,                    // 0xFFFF
    if_: u8,                   // 0xFF0F
    boot_rom: Option<Vec<u8>>, // Boot ROM (0x100 bytes)
    boot_rom_enabled: bool,    // FF50 controle

    // ===== OAM DMA =====
    oam_dma_active: bool,
    oam_dma_src: u16,
    oam_dma_index: u8,
    oam_dma_cycles: u32,

    // ===== Serial =====
    serial_sb: u8, // FF01
    serial_sc: u8, // FF02
}

impl MemoryBus {
    /// Carrega a boot ROM (256 bytes) e ativa mapeamento.
    pub fn load_boot_rom(&mut self, data: Vec<u8>) {
        if data.len() == 0x100 {
            self.boot_rom = Some(data);
            self.boot_rom_enabled = true;
        } else {
            eprintln!("Boot ROM inválida: esperado 256 bytes");
        }
    }

    #[inline]
    pub fn get_ie(&self) -> u8 {
        self.ie
    }

    #[inline]
    pub fn get_if(&self) -> u8 {
        self.if_
    }

    /// Limpa bits específicos de IF e reflete no registrador mapeado em 0xFF0F
    #[inline]
    pub fn clear_if_bits(&mut self, mask: u8) {
        self.if_ &= !mask;
        self.write(0xFF0F, self.if_);
    }

    /// Seta o bit de interrupção do Joypad (IF bit 4)
    pub fn request_joypad_interrupt(&mut self) {
        self.if_ |= 0x10;
    }

    pub fn load_cart_ram(&mut self, path: &str) -> Result<(), String> {
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        self.mbc.load_ram(&data);
        Ok(())
    }

    pub fn save_cart_ram(&self, path: &str) -> Result<(), String> {
        if let Some(data) = self.mbc.save_ram() {
            std::fs::write(path, &data).map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("No RAM to save".to_string())
        }
    }

    pub fn new(mbc: Box<dyn MBC>) -> Self {
        let mut rng = rand::thread_rng();
        let mut wram = [0u8; 0x2000];
        let mut hram = [0u8; 0x7F];
        rng.fill(&mut wram[..]);
        rng.fill(&mut hram[..]);
        Self {
            mbc,
            wram,
            hram,
            timer: Timer::new(),
            joypad: Joypad::new(),
            ppu: PPU::PPU::new(),
            apu: APU::APU::new(),
            tima: 0,
            tma: 0,
            tac: 0,
            ie: 0,
            if_: 0,
            boot_rom: None, // Boot ROM (0x100 bytes)
            boot_rom_enabled: false,
            oam_dma_active: false,
            oam_dma_src: 0,
            oam_dma_index: 0,
            oam_dma_cycles: 0,
            serial_sb: 0x00,
            serial_sc: 0x7E, // bits não usados em 1
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        // Boot ROM mapeada em 0x0000–0x00FF enquanto boot_rom_enabled
        if address <= 0x00FF && self.boot_rom_enabled {
            if let Some(ref rom) = self.boot_rom {
                return rom[address as usize];
            }
        }

        match address {
            0x0000..=0x7FFF => self.mbc.read_rom(address),
            0x8000..=0x9FFF => self.ppu.read_vram(address),
            0xA000..=0xBFFF => self.mbc.read_ram(address),
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
            0xFE00..=0xFE9F => self.ppu.read_oam(address),
            0xFF00 => self.joypad.read(),
            0xFF01 => self.serial_sb,
            0xFF02 => self.serial_sc | 0b0111_1110,
            0xFF04 => self.timer.read_div(),
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            0xFF0F => self.if_,
            0xFF10..=0xFF3F => self.apu.read_register(address),
            0xFF40..=0xFF4B => self.ppu.read_register(address),
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.ie,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if address == 0xFF50 {
            if self.boot_rom_enabled && (value & 0x01) != 0 {
                self.boot_rom_enabled = false;
            }
            return;
        }

        // OAM DMA: escrever em FF46 inicia transferência
        if address == 0xFF46 {
            self.start_oam_dma(value);
            return;
        }

        match address {
            0x0000..=0x7FFF => self.mbc.write_register(address, value),
            0x8000..=0x9FFF => self.ppu.write_vram(address, value),
            0xA000..=0xBFFF => self.mbc.write_ram(address, value),
            0xC000..=0xDFFF => {
                let idx = (address - 0xC000) as usize;
                self.wram[idx] = value;
                // Espelha na echo RAM
                let echo_addr = address + 0x2000;
                if echo_addr <= 0xFDFF {
                    self.wram[(echo_addr - 0xE000) as usize] = value;
                }
            }
            0xE000..=0xFDFF => {
                let idx = (address - 0xE000) as usize;
                self.wram[idx] = value;
                // Espelha na WRAM principal
                let main_addr = address - 0x2000;
                if main_addr >= 0xC000 && main_addr <= 0xDFFF {
                    self.wram[(main_addr - 0xC000) as usize] = value;
                }
            }
            0xFE00..=0xFE9F => self.ppu.write_oam(address, value),
            0xFF00 => self.joypad.write(value),
            0xFF01 => self.serial_sb = value,
            0xFF02 => self.serial_sc = value & 0b1000_0001,
            0xFF04 => {
                let (new_tima, new_if) = self
                    .timer
                    .reset_div(self.tima, self.tma, self.tac, self.if_);
                self.tima = new_tima;
                self.if_ = new_if;
            }
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => {
                let (new_tima, new_if) = self
                    .timer
                    .write_tac(self.tima, self.tma, self.tac, value, self.if_);
                self.tima = new_tima;
                self.if_ = new_if;
                self.tac = value;
            }
            0xFF0F => self.if_ = value,
            0xFF10..=0xFF3F => self.apu.write_register(address, value),
            0xFF40..=0xFF4B => self.ppu.write_register(address, value, &mut self.if_),
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.ie = value,
            _ => {}
        }
    }

    /// Inicia uma transferência OAM DMA a partir de `value << 8`
    pub fn start_oam_dma(&mut self, value: u8) {
        let src = (value as u16) << 8;
        self.oam_dma_src = src;
        self.oam_dma_index = 0;
        self.oam_dma_cycles = 0;
        self.oam_dma_active = true;
    }

    /// Lê um byte da fonte do DMA sem causar efeitos colaterais extras.
    fn oam_dma_read_source(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.mbc.read_rom(addr),
            0x8000..=0x9FFF => self.ppu.read_vram(addr),
            0xA000..=0xBFFF => self.mbc.read_ram(addr),
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],
            0xE000..=0xFDFF => {
                let base = addr - 0x2000;
                if (0xC000..=0xDDFF).contains(&base) {
                    self.wram[(base - 0xC000) as usize]
                } else {
                    0xFF
                }
            }
            _ => 0xFF,
        }
    }

    /// Avança OAM DMA consumindo `cycles` da CPU.
    fn step_oam_dma(&mut self, cycles: u32) {
        if !self.oam_dma_active {
            return;
        }
        self.oam_dma_cycles = self.oam_dma_cycles.saturating_add(cycles);
        while self.oam_dma_cycles >= 4 && self.oam_dma_index < 160 {
            self.oam_dma_cycles -= 4;
            let src_addr = self.oam_dma_src.wrapping_add(self.oam_dma_index as u16);
            let val = self.oam_dma_read_source(src_addr);
            let dst_addr = 0xFE00u16 + self.oam_dma_index as u16;
            self.ppu.write_oam(dst_addr, val);
            self.oam_dma_index = self.oam_dma_index.wrapping_add(1);
        }
        if self.oam_dma_index >= 160 {
            self.oam_dma_active = false;
        }
    }

    pub fn tick(&mut self, cycles: u32) {
        self.step_oam_dma(cycles);
        let (new_tima, new_if) = self
            .timer
            .tick(cycles, self.tima, self.tma, self.tac, self.if_);
        self.tima = new_tima;
        self.if_ = new_if;
        for _ in 0..cycles {
            self.apu.tick();
        }
        self.ppu.step(cycles, &mut self.if_);
    }
}
