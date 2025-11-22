
use crate::GB::mbc::MBC;
use crate::GB::PPU;
use crate::GB::APU;

pub struct RAM {
    memory: [u8; 65536],
    mbc: Box<dyn MBC>,
    pub ppu: PPU::PPU,
    pub apu: APU::APU,
    pub trace_enabled: bool,
    // Joypad state
    joypad_dpad: u8,
    joypad_buttons: u8,
    // Timer state
    div_counter: u16,
    timer_last_signal: bool,
    tima_reload_delay: u8,
}

impl RAM {
    // DMA transfer: copia 160 bytes para OAM
    fn dma_transfer(&mut self, value: u8) {
        let source = (value as u16) << 8;
        for offset in 0..160u16 {
            let byte = self.read(source + offset);
            self.ppu.write_oam(0xFE00 + offset, byte);
        }
    }

    // === API pública para manipular joypad ===

    pub fn press_joypad_button(&mut self, button: u8, is_dpad: bool) {
        if is_dpad {
            self.joypad_dpad &= !(1 << button);
        } else {
            self.joypad_buttons &= !(1 << button);
        }
    }

    pub fn release_joypad_button(&mut self, button: u8, is_dpad: bool) {
        if is_dpad {
            self.joypad_dpad |= 1 << button;
        } else {
            self.joypad_buttons |= 1 << button;
        }
    }

    // Timer helpers
    fn timer_bit_index(&self) -> u8 {
        match self.memory[0xFF07] & 0x03 {
            0b00 => 9,
            0b01 => 3,
            0b10 => 5,
            _ => 7,
        }
    }

    fn timer_enabled(&self) -> bool {
        (self.memory[0xFF07] & 0x04) != 0
    }

    fn current_timer_signal(&self) -> bool {
        let enabled = self.timer_enabled();
        if !enabled { return false; }
        let bit = self.timer_bit_index();
        ((self.div_counter >> bit) & 1) != 0
    }

    pub fn tick_timers(&mut self, cycles: u32) {
        for _ in 0..cycles {
            self.div_counter = self.div_counter.wrapping_add(1);
            self.memory[0xFF04] = (self.div_counter >> 8) as u8;
            if self.tima_reload_delay > 0 {
                self.tima_reload_delay -= 1;
                if self.tima_reload_delay == 0 {
                    let tma = self.memory[0xFF06];
                    self.memory[0xFF05] = tma;
                    self.memory[0xFF0F] |= 0x04;
                }
            }
            let signal = self.current_timer_signal();
            if self.timer_last_signal && !signal {
                let tima = self.memory[0xFF05];
                if tima == 0xFF {
                    self.memory[0xFF05] = 0x00;
                    self.tima_reload_delay = 4;
                } else {
                    self.memory[0xFF05] = tima.wrapping_add(1);
                }
            }
            self.timer_last_signal = signal;
        }
    }
    pub fn new(mbc: Box<dyn MBC>) -> Self {
        RAM {
            memory: [0; 65536],
            mbc,
            ppu: PPU::PPU::new(),
            apu: APU::APU::new(),
            trace_enabled: false,
            joypad_dpad: 0xFF,
            joypad_buttons: 0xFF,
            div_counter: 0,
            timer_last_signal: false,
            tima_reload_delay: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        if address == 0xFF00 {
            // Joypad register
            let select = self.memory[0xFF00];
            let mut result = select & 0xF0;
            // Bit 4 = D-pad select (0 = selected)
            // Bit 5 = Button select (0 = selected)
            if select & 0x10 == 0 {
                // D-pad selected
                result |= self.joypad_dpad & 0x0F;
            } else if select & 0x20 == 0 {
                // Button group selected
                result |= self.joypad_buttons & 0x0F;
            } else {
                // Nenhum grupo selecionado: bits baixos = 0x0F
                result |= 0x0F;
            }
            result
        } else {
            match address {
                0x0000..=0x7FFF => self.mbc.read_rom(address),
                0x8000..=0x9FFF => self.ppu.read_vram(address),
                0xA000..=0xBFFF => self.mbc.read_ram(address),
                0xFE00..=0xFE9F => self.ppu.read_oam(address),
                0xFF0F => self.memory[0xFF0F], // IF
                0xFFFF => self.memory[0xFFFF], // IE
                0xFF10..=0xFF3F => self.apu.read_register(address),
                0xFF40..=0xFF4B => self.ppu.read_register(address),
                _ => self.memory[address as usize],
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => {
                self.mbc.write_register(address, value);
            }
            0x8000..=0x9FFF => {
                self.ppu.write_vram(address, value);
            }
            0xA000..=0xBFFF => {
                self.mbc.write_ram(address, value);
            }
            0xFE00..=0xFE9F => {
                self.ppu.write_oam(address, value);
            }
            // DMA transfer (OAM sprites)
            0xFF46 => {
                self.memory[0xFF46] = value;
                self.dma_transfer(value);
            }
            // Joypad select
            0xFF00 => {
                self.memory[0xFF00] = value & 0x30;
            }
            0xFF0F => {
                self.memory[0xFF0F] = value;
            }
            0xFFFF => {
                self.memory[0xFFFF] = value;
            }
            0xFF10..=0xFF3F => {
                self.apu.write_register(address, value);
            }
            0xFF40..=0xFF4B => {
                self.ppu.write_register(address, value);
            }
            // Demais registradores / RAM
            _ => {
                self.memory[address as usize] = value;
            }
        }
    }


    // === Save/Load persistence para arquivos .sav ===

    /// Salva a RAM do cartucho para arquivo .sav
    pub fn save_cart_ram(&self, sav_path: &str) -> Result<(), std::io::Error> {
        use std::fs;
        if let Some(data) = self.mbc.save_ram() {
            if !data.is_empty() {
                fs::write(sav_path, &data)?;
                println!("💾 Save criado: {} ({} bytes)", sav_path, data.len());
            }
        }
        Ok(())
    }

    /// Carrega a RAM do cartucho de arquivo .sav
    pub fn load_cart_ram(&mut self, sav_path: &str) -> Result<(), std::io::Error> {
        use std::fs;
        if !std::path::Path::new(sav_path).exists() {
            println!("📁 Nenhum save encontrado: {}", sav_path);
            return Ok(());
        }
        let save_data = fs::read(sav_path)?;
        if !save_data.is_empty() {
            self.mbc.load_ram(&save_data);
            println!("💾 Save carregado: {} ({} bytes)", sav_path, save_data.len());
        }
        Ok(())
    }

    /// Avança PPU e aplica as mudanças de IF (0xFF0F)
    pub fn step_ppu(&mut self, cycles: u32) {
        let mut iflags = self.read(0xFF0F);
        self.ppu.step(cycles, &mut iflags);
        self.write(0xFF0F, iflags);
    }
}