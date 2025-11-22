
use crate::GB::mbc::MBC;
use crate::GB::PPU;
use crate::GB::APU;
use crate::GB::timer::Timer;

pub struct RAM {
    memory: [u8; 65536],
    mbc: Box<dyn MBC>,
    pub ppu: PPU::PPU,
    pub apu: APU::APU,
    pub trace_enabled: bool,
    pub joypad: crate::GB::joypad::Joypad,
    pub timer: Timer,
}

impl RAM {
    // DMA transfer: copia 160 bytes para OAM
    /// Executa transferência DMA instantânea para OAM.
    /// Lê diretamente da ROM, VRAM, RAM do cartucho, RAM interna e echo RAM.
    /// Não lê da OAM nem da área proibida.
    fn dma_transfer(&mut self, value: u8) {
        let base = (value as u16) << 8;
        for offset in 0..160u16 {
            let addr = base + offset;
            let byte = match addr {
                0x0000..=0x7FFF => self.mbc.read_rom(addr),
                0x8000..=0x9FFF => self.ppu.read_vram(addr),
                0xA000..=0xBFFF => self.mbc.read_ram(addr),
                0xC000..=0xDFFF => self.memory[addr as usize],
                0xE000..=0xFDFF => self.memory[(addr - 0x2000) as usize], // echo RAM
                _ => 0xFF, // áreas proibidas retornam 0xFF
            };
            self.ppu.write_oam(0xFE00 + offset, byte);
        }
    }

    pub fn tick_timers(&mut self, cycles: u32) {
        let tma = self.memory[0xFF06];
        let tac = self.memory[0xFF07];
        let tima = self.memory[0xFF05];
        let if_reg = self.memory[0xFF0F];
        let (new_tima, new_if) = self.timer.tick(cycles, tima, tma, tac, if_reg);
        self.memory[0xFF05] = new_tima;
        self.memory[0xFF0F] = new_if;
        self.memory[0xFF04] = self.timer.read_div();
    }

    pub fn new(mbc: Box<dyn MBC>) -> Self {
        RAM {
            memory: [0; 65536],
            mbc,
            ppu: PPU::PPU::new(),
            apu: APU::APU::new(),
            trace_enabled: false,
            joypad: crate::GB::joypad::Joypad::new(),
            timer: Timer::new(),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        if address == 0xFF00 {
            return self.joypad.read();
        }
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
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => self.mbc.write_register(address, value),
            0x8000..=0x9FFF => self.ppu.write_vram(address, value),
            0xA000..=0xBFFF => self.mbc.write_ram(address, value),
            0xFE00..=0xFE9F => self.ppu.write_oam(address, value),
            0xFF00 => self.joypad.write(value),
            0xFF04 => {
                let tima = self.memory[0xFF05];
                let tma = self.memory[0xFF06];
                let tac = self.memory[0xFF07];
                let if_reg = self.memory[0xFF0F];
                let (new_tima, new_if) = self.timer.reset_div(tima, tma, tac, if_reg);
                self.memory[0xFF05] = new_tima;
                self.memory[0xFF0F] = new_if;
                self.memory[0xFF04] = 0;
            },
            0xFF07 => {
                let tima = self.memory[0xFF05];
                let tma = self.memory[0xFF06];
                let old_tac = self.memory[0xFF07];
                let if_reg = self.memory[0xFF0F];
                let (new_tima, new_if) = self.timer.write_tac(tima, tma, old_tac, value, if_reg);
                self.memory[0xFF05] = new_tima;
                self.memory[0xFF0F] = new_if;
                self.memory[0xFF07] = value;
            },
            0xFF46 => {
                self.dma_transfer(value);
            },
            0xFF0F => self.memory[0xFF0F] = value, // IF
            0xFFFF => self.memory[0xFFFF] = value, // IE
            0xFF10..=0xFF3F => self.apu.write_register(address, value),
            0xFF40..=0xFF4B => self.ppu.write_register(address, value),
            _ => self.memory[address as usize] = value,
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