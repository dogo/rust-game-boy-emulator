use crate::GB::mbc::MBC;
use crate::GB::timer::Timer;
use crate::GB::joypad::Joypad;
use crate::GB::PPU;
use crate::GB::APU;

pub struct MemoryBus {
    mbc: Box<dyn MBC>,
    wram: [u8; 0x2000],  // Work RAM (8KB)
    hram: [u8; 0x7F],    // High RAM (127 bytes)
    timer: Timer,
    pub joypad: Joypad,
    pub ppu: PPU::PPU,
    pub apu: APU::APU,
    tima: u8, // FF05
    tma: u8,  // FF06
    tac: u8,  // FF07
    ie: u8,  // 0xFFFF
    if_: u8, // 0xFF0F
}

impl MemoryBus {
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
        let joypad = Joypad::new();
        let mut bus = Self {
            mbc,
            wram: [0; 0x2000],
            hram: [0; 0x7F],
            timer: Timer::new(),
            joypad,
            ppu: PPU::PPU::new(),
            apu: APU::APU::new(),
            tima: 0,
            tma: 0,
            tac: 0,
            ie: 0,
            if_: 0,
        };
        // Conecta o callback de interrupção do joypad
        let if_ptr: *mut u8 = &mut bus.if_;
        bus.joypad.request_interrupt = Some(Box::new(move || unsafe {
            *if_ptr |= 0x10; // bit 4 - joypad
        }));
        bus
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.mbc.read_rom(address),
            0x8000..=0x9FFF => self.ppu.read_vram(address),
            0xA000..=0xBFFF => self.mbc.read_ram(address),
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
            0xFE00..=0xFE9F => self.ppu.read_oam(address),
            0xFF00 => self.joypad.read(),
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
            },
            0xE000..=0xFDFF => {
                let idx = (address - 0xE000) as usize;
                self.wram[idx] = value;
                // Espelha na WRAM principal
                let main_addr = address - 0x2000;
                if main_addr >= 0xC000 && main_addr <= 0xDFFF {
                    self.wram[(main_addr - 0xC000) as usize] = value;
                }
            },
            0xFE00..=0xFE9F => self.ppu.write_oam(address, value),
            0xFF46 => self.dma_transfer(value),
            0xFF00 => self.joypad.write(value),
            0xFF04 => {
                let (new_tima, new_if) = self.timer.reset_div(self.tima, self.tma, self.tac, self.if_);
                self.tima = new_tima;
                self.if_ = new_if;
            },
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => {
                let (new_tima, new_if) = self.timer.write_tac(self.tima, self.tma, self.tac, value, self.if_);
                self.tima = new_tima;
                self.if_ = new_if;
                self.tac = value;
            },
            0xFF0F => self.if_ = value,
            0xFF10..=0xFF3F => self.apu.write_register(address, value),
            0xFF40..=0xFF4B => self.ppu.write_register(address, value),
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.ie = value,
            _ => {},
        }
    }

    /// DMA instantâneo, leitura crua sem OAM, echo RAM, e áreas proibidas retornam 0xFF
    fn dma_transfer(&mut self, value: u8) {
        let base = (value as u16) << 8;
        for offset in 0..160u16 {
            let addr = base + offset;
            let byte = match addr {
                0x0000..=0x7FFF => self.mbc.read_rom(addr),
                0x8000..=0x9FFF => self.ppu.read_vram(addr),
                0xA000..=0xBFFF => self.mbc.read_ram(addr),
                0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],
                0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize], // echo RAM
                _ => 0xFF, // áreas proibidas retornam 0xFF
            };
            self.ppu.write_oam(0xFE00 + offset, byte);
        }
    }

    pub fn tick(&mut self, cycles: u32) {
        let (new_tima, new_if) = self.timer.tick(cycles, self.tima, self.tma, self.tac, self.if_);
        self.tima = new_tima;
        self.if_ = new_if;
        // Tick do APU por ciclo
        for _ in 0..cycles {
            self.apu.tick();
        }
        self.ppu.step(cycles, &mut self.if_);
    }
}
