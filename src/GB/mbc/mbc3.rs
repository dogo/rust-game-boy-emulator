use std::time::{SystemTime, UNIX_EPOCH};

use super::MBC;

pub struct MBC3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,

    // RTC regs: [sec, min, hour, day_low, day_high]
    // day_high: bit0 = day bit8, bit6 = HALT, bit7 = carry
    rtc: [u8; 5],
    rtc_latch: [u8; 5],
    rtc_latch_state: u8,

    // Timestamp do host (segundos desde UNIX_EPOCH) para avanço do RTC
    rtc_last_update: i64,
}

impl MBC3 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {
        let now = Self::now_secs();
        Self {
            rom,
            ram: vec![0; ram_size],
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            rtc: [0; 5],
            rtc_latch: [0; 5],
            rtc_latch_state: 0,
            rtc_last_update: now,
        }
    }

    #[inline]
    fn now_secs() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    /// Avança o RTC com base no tempo do host.
    fn update_rtc(&mut self) {
        let now = Self::now_secs();

        // Primeira atualização: só ancora o relógio
        if self.rtc_last_update == 0 {
            self.rtc_last_update = now;
            return;
        }

        let delta = now.saturating_sub(self.rtc_last_update);
        if delta <= 0 {
            return;
        }

        // Se HALT está setado, o RTC não avança, só atualiza âncora
        if (self.rtc[4] & 0x40) != 0 {
            self.rtc_last_update = now;
            return;
        }

        self.add_rtc_seconds(delta as u64);
        self.rtc_last_update = now;
    }

    /// Soma segundos ao RTC sem iterar por cada minuto passado.
    fn add_rtc_seconds(&mut self, seconds: u64) {
        let day = (((self.rtc[4] & 1) as u128) << 8) | self.rtc[3] as u128;
        let total = day * 86_400
            + self.rtc[2] as u128 * 3_600
            + self.rtc[1] as u128 * 60
            + self.rtc[0] as u128
            + seconds as u128;
        let days = total / 86_400;
        let remaining = total % 86_400;
        self.rtc[0] = (remaining % 60) as u8;
        self.rtc[1] = ((remaining / 60) % 60) as u8;
        self.rtc[2] = (remaining / 3_600) as u8;
        let day = days % 512;
        self.rtc[3] = day as u8;
        self.rtc[4] = (self.rtc[4] & 0xC0) | ((day >> 8) as u8);
        if days >= 512 {
            self.rtc[4] |= 0x80;
        }
    }
}

impl MBC for MBC3 {
    fn ram(&self) -> &[u8] {
        &self.ram
    }

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
            // RAM / RTC enable
            0x0000..=0x1FFF => self.ram_enabled = (value & 0x0F) == 0x0A,

            // ROM bank select
            0x2000..=0x3FFF => {
                let mut bank = value & 0x7F;
                if bank == 0 {
                    bank = 1;
                }
                self.rom_bank = bank;
            }

            // RAM bank / RTC register select
            0x4000..=0x5FFF => {
                self.ram_bank = value;
            }

            // RTC latch
            0x6000..=0x7FFF => {
                // 0 → 1: latch
                if self.rtc_latch_state == 0x00 && value == 0x01 {
                    // Atualiza o RTC antes de latchear
                    self.update_rtc();
                    self.rtc_latch.copy_from_slice(&self.rtc);
                }
                self.rtc_latch_state = value;
            }
            _ => {}
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }

        match self.ram_bank {
            // RAM normal
            0x00..=0x03 => {
                let idx = (self.ram_bank as usize) * 0x2000 + ((address - 0xA000) as usize);
                self.ram.get(idx).copied().unwrap_or(0xFF)
            }

            // RTC latched regs
            0x08..=0x0C => self.rtc_latch[(self.ram_bank - 0x08) as usize],

            _ => 0xFF,
        }
    }

    fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }

        // Antes de mexer em RAM/RTC, atualiza o relógio
        self.update_rtc();

        match self.ram_bank {
            // RAM normal
            0x00..=0x03 => {
                let idx = (self.ram_bank as usize) * 0x2000 + ((address - 0xA000) as usize);
                if idx < self.ram.len() {
                    self.ram[idx] = value;
                }
            }

            // RTC registers
            0x08..=0x0C => {
                let reg = (self.ram_bank - 0x08) as usize;
                match reg {
                    // shortcut: valores inválidos usam módulo 60/24; ajustar se testes em hardware indicarem outra regra.
                    0 | 1 => self.rtc[reg] = value % 60,
                    2 => self.rtc[reg] = value % 24,
                    3 => self.rtc[reg] = value,
                    // day high (bit0=day8, bit6=HALT, bit7=carry)
                    4 => {
                        // Só bits 0,6,7 são usados; preserva o resto como está
                        let old = self.rtc[4];
                        let mut new = old;

                        // atualiza bit0 (day high)
                        new = (new & !0x01) | (value & 0x01);

                        // atualiza HALT (bit6)
                        new = (new & !0x40) | (value & 0x40);

                        // escrever carry (bit7) permite limpar o carry (o jogo faz isso)
                        new = (new & !0x80) | (value & 0x80);

                        self.rtc[4] = new;

                        // Se acabamos de sair de HALT, ressincroniza a âncora
                        if (old & 0x40) != 0 && (new & 0x40) == 0 {
                            self.rtc_last_update = Self::now_secs();
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn save_ram(&self) -> Option<Vec<u8>> {
        let has_rtc = matches!(self.rom.get(0x0147).copied(), Some(0x0F | 0x10));
        if !super::rom_has_battery(&self.rom) || (self.ram.is_empty() && !has_rtc) {
            None
        } else {
            // Cria uma cópia mutável para atualizarmos o RTC antes de salvar
            let mut clone = self.clone_for_save();
            clone.update_rtc();

            let mut buf = clone.ram.clone();
            // Salva os 5 regs do RTC
            buf.extend_from_slice(&clone.rtc);
            // Salva o timestamp do host (i64 little endian)
            buf.extend_from_slice(&clone.rtc_last_update.to_le_bytes());
            Some(buf)
        }
    }

    fn load_ram(&mut self, data: &[u8]) -> Result<(), String> {
        // Sem bateria o emulador nunca grava save, então um arquivo existente é ignorado
        if !super::rom_has_battery(&self.rom) {
            return Ok(());
        }
        let ram_len = self.ram.len();
        let has_rtc = data.len() == ram_len + 5;
        let has_timestamp = data.len() == ram_len + 13;
        if data.is_empty() || (data.len() != ram_len && !has_rtc && !has_timestamp) {
            return Err(format!(
                "tamanho inválido do save MBC3: {} bytes",
                data.len()
            ));
        }

        let rtc = if has_rtc || has_timestamp {
            let rtc: [u8; 5] = data[ram_len..ram_len + 5].try_into().unwrap();
            if rtc[0] >= 60 || rtc[1] >= 60 || rtc[2] >= 24 || rtc[4] & !0xC1 != 0 {
                return Err("registradores RTC inválidos no save".to_string());
            }
            Some(rtc)
        } else {
            None
        };

        let saved_ts = if has_timestamp {
            let timestamp = i64::from_le_bytes(data[ram_len + 5..].try_into().unwrap());
            if timestamp <= 0 {
                return Err("timestamp RTC inválido no save".to_string());
            }
            Some(timestamp)
        } else {
            None
        };

        let now = Self::now_secs();
        self.ram.copy_from_slice(&data[..ram_len]);
        if let Some(rtc) = rtc {
            self.rtc = rtc;
        }
        if let Some(timestamp) = saved_ts {
            if now > timestamp && (self.rtc[4] & 0x40) == 0 {
                self.add_rtc_seconds((now - timestamp) as u64);
            }
        }
        self.rtc_last_update = now;
        self.rtc_latch.copy_from_slice(&self.rtc);
        Ok(())
    }
}

impl MBC3 {
    /// Helper para `save_ram`: clona os campos necessários sem exigir Clone completo em MBC3.
    fn clone_for_save(&self) -> MBC3 {
        MBC3 {
            rom: Vec::new(), // ROM não é usada no save_ram
            ram: self.ram.clone(),
            ram_enabled: self.ram_enabled,
            rom_bank: self.rom_bank,
            ram_bank: self.ram_bank,
            rtc: self.rtc,
            rtc_latch: self.rtc_latch,
            rtc_latch_state: self.rtc_latch_state,
            rtc_last_update: self.rtc_last_update,
        }
    }
}
