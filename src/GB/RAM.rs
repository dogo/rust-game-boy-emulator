pub struct RAM {
    // todas as 65.536 posições endereçáveis
    memory: [u8; 65536],
    // ROM completa do cartucho (para mapeamento por banco)
    rom: Vec<u8>,
    cart_type: u8,
    // Estado MBC (mínimo para MBC3)
    rom_bank: u8,       // banco ROM selecionado para 0x4000..=0x7FFF (em MBC3, 1..=0x7F)
    ram_enabled: bool,  // enable RAM/RTC (0x0000..=0x1FFF)
    ram_bank: u8,       // seleção de banco de RAM ou registrador RTC (0x4000..=0x5FFF)
    // RAM externa do cartucho (MBC3: até 4 bancos de 8KB = 32KB)
    cart_ram: Vec<u8>,
    // RTC (Real Time Clock) do MBC3
    rtc_s: u8,          // segundos (0..59)
    rtc_m: u8,          // minutos (0..59)
    rtc_h: u8,          // horas (0..23)
    rtc_dl: u8,         // dia baixo (bits 0..7 de day counter)
    rtc_dh: u8,         // dia alto + flags (bit 0 = bit 8 de day, bit 6 = halt, bit 7 = carry)
    rtc_latched_s: u8,
    rtc_latched_m: u8,
    rtc_latched_h: u8,
    rtc_latched_dl: u8,
    rtc_latched_dh: u8,
    rtc_latch_state: u8, // estado do latch (0x00 ou 0x01 para detectar transição 0->1)
    // Estado interno do temporizador
    div_counter: u16,        // contador divisor interno de 16 bits (incrementa a cada ciclo da CPU)
    timer_last_signal: bool, // último nível do sinal do timer (enable && bit selecionado de div_counter)
    tima_reload_delay: u8,   // se >0, contagem regressiva para recarregar TIMA com TMA e solicitar interrupção
}

impl RAM {
    pub fn new() -> Self {
        RAM {
            memory: [0; 65536],
            rom: Vec::new(),
            cart_type: 0x00,
            rom_bank: 1,
            ram_enabled: false,
            ram_bank: 0,
            cart_ram: vec![0; 32 * 1024], // 32KB = 4 bancos de 8KB
            rtc_s: 0,
            rtc_m: 0,
            rtc_h: 0,
            rtc_dl: 0,
            rtc_dh: 0,
            rtc_latched_s: 0,
            rtc_latched_m: 0,
            rtc_latched_h: 0,
            rtc_latched_dl: 0,
            rtc_latched_dh: 0,
            rtc_latch_state: 0,
            div_counter: 0,
            timer_last_signal: false,
            tima_reload_delay: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        let addr = address as usize;
        // Mapeamento de ROM (0x0000..=0x7FFF)
        if addr < 0x8000 {
            if self.is_mbc3() {
                if addr < 0x4000 {
                    return self.rom_get(addr);
                } else {
                    let off = addr - 0x4000;
                    let bank = (self.rom_bank as usize).max(1);
                    let idx = bank * 0x4000 + off;
                    return self.rom_get(idx);
                }
            } else {
                // ROM ONLY / sem MBC: banco 0 fixo
                return self.rom_get(addr);
            }
        }
        // Mapeamento de RAM externa do cartucho (0xA000..=0xBFFF)
        if addr >= 0xA000 && addr < 0xC000 {
            if self.is_mbc3() && self.ram_enabled {
                if self.ram_bank <= 0x03 {
                    // RAM banco 0..3
                    let ram_addr = (self.ram_bank as usize) * 0x2000 + (addr - 0xA000);
                    if ram_addr < self.cart_ram.len() {
                        return self.cart_ram[ram_addr];
                    }
                } else if self.ram_bank >= 0x08 && self.ram_bank <= 0x0C {
                    // RTC register latch read
                    return match self.ram_bank {
                        0x08 => self.rtc_latched_s,
                        0x09 => self.rtc_latched_m,
                        0x0A => self.rtc_latched_h,
                        0x0B => self.rtc_latched_dl,
                        0x0C => self.rtc_latched_dh,
                        _ => 0xFF,
                    };
                }
            }
            return 0xFF; // RAM desabilitada ou endereço inválido
        }
        self.memory[addr]
    }

    pub fn write(&mut self, address: u16, byte: u8) {
        // Tratamento de registradores MBC3 (mínimo)
        if self.is_mbc3() {
            match address {
                0x0000..=0x1FFF => {
                    // Enable/disable RAM/RTC (0x0A habilita, outros desabilitam)
                    let old = self.ram_enabled;
                    self.ram_enabled = (byte & 0x0F) == 0x0A;
                    if old != self.ram_enabled {
                        println!("[MBC3] RAM/RTC {}", if self.ram_enabled { "habilitado" } else { "desabilitado" });
                    }
                    return;
                }
                0x2000..=0x3FFF => {
                    // Seleção de banco ROM (7 bits, 0 => 1)
                    let mut bank = byte & 0x7F;
                    if bank == 0 { bank = 1; }
                    if self.rom_bank != bank {
                        println!("[MBC3] Banco ROM: {:02X} -> {:02X}", self.rom_bank, bank);
                        self.rom_bank = bank;
                    } else {
                        self.rom_bank = bank;
                    }
                    return;
                }
                0x4000..=0x5FFF => {
                    // Seleção de banco RAM (00-03) ou registrador RTC (08-0C)
                    if self.ram_bank != byte {
                        let desc = if byte <= 0x03 {
                            format!("RAM banco {:02X}", byte)
                        } else if byte >= 0x08 && byte <= 0x0C {
                            format!("RTC reg {:02X}", byte)
                        } else {
                            format!("valor {:02X}", byte)
                        };
                        println!("[MBC3] Seleção RAM/RTC: {}", desc);
                        self.ram_bank = byte;
                    } else {
                        self.ram_bank = byte;
                    }
                    return;
                }
                0x6000..=0x7FFF => {
                    // Latch clock: transição 0x00 -> 0x01 captura RTC atual
                    if self.rtc_latch_state == 0x00 && byte == 0x01 {
                        self.rtc_latched_s = self.rtc_s;
                        self.rtc_latched_m = self.rtc_m;
                        self.rtc_latched_h = self.rtc_h;
                        self.rtc_latched_dl = self.rtc_dl;
                        self.rtc_latched_dh = self.rtc_dh;
                        println!("[MBC3] RTC latched: {:02}:{:02}:{:02} dia={}",
                                 self.rtc_h, self.rtc_m, self.rtc_s,
                                 ((self.rtc_dh as u16 & 1) << 8) | self.rtc_dl as u16);
                    }
                    self.rtc_latch_state = byte;
                    return;
                }
                _ => {}
            }
        }
        match address {
            0xFF04 => { // escrita em DIV: zera o divisor interno e o registrador DIV
                self.div_counter = 0;
                self.memory[address as usize] = 0;
                // Recalcula o último sinal do timer após o reset para evitar bordas espúrias
                self.timer_last_signal = self.current_timer_signal();
                println!("[TIMER] DIV<=00 (reset)");
            }
            0xFF07 => { // TAC
                self.memory[address as usize] = byte & 0x07; // apenas os 3 bits menos significativos são usados
                // Atualiza o último sinal conforme a nova configuração do TAC
                self.timer_last_signal = self.current_timer_signal();
                let en = (byte & 0x04) != 0;
                let freq = match byte & 0x03 { 0b00 => 4096, 0b01 => 262144, 0b10 => 65536, _ => 16384 };
                println!("[TIMER] TAC<={:02X} (enable={}, freq={}Hz)", byte & 0x07, en as u8, freq);
            }
            0xFF05 | 0xFF06 => { // TIMA, TMA
                self.memory[address as usize] = byte;
                if address == 0xFF05 {
                    println!("[TIMER] TIMA<={:02X}", byte);
                } else {
                    println!("[TIMER] TMA<={:02X}", byte);
                }
            }
            0xA000..=0xBFFF => {
                // Escrita em RAM externa ou RTC
                if self.is_mbc3() && self.ram_enabled {
                    if self.ram_bank <= 0x03 {
                        // RAM banco 0..3
                        let ram_addr = (self.ram_bank as usize) * 0x2000 + (address as usize - 0xA000);
                        if ram_addr < self.cart_ram.len() {
                            self.cart_ram[ram_addr] = byte;
                        }
                    } else if self.ram_bank >= 0x08 && self.ram_bank <= 0x0C {
                        // Escrita nos registradores RTC
                        match self.ram_bank {
                            0x08 => self.rtc_s = byte & 0x3F,   // 0..59
                            0x09 => self.rtc_m = byte & 0x3F,   // 0..59
                            0x0A => self.rtc_h = byte & 0x1F,   // 0..23
                            0x0B => self.rtc_dl = byte,
                            0x0C => self.rtc_dh = byte,
                            _ => {}
                        }
                    }
                }
                return;
            }
            _ => {
                // Evita escrita em ROM (0x0000..=0x7FFF)
                if address < 0x8000 { return; }
                self.memory[address as usize] = byte;
            }
        }
    }

    pub fn load_bytes(&mut self, data: &[u8]) {
        // Armazena ROM completa e define tipo de cartucho
        self.rom = data.to_vec();
        self.cart_type = if self.rom.len() > 0x0147 { self.rom[0x0147] } else { 0x00 };
        self.rom_bank = 1;
        // Limpa a RAM interna (áreas não-ROM)
        self.memory = [0; 65536];
    }

    // Auxiliares do temporizador
    fn timer_bit_index(&self) -> u8 {
        match self.memory[0xFF07] & 0x03 { // TAC[1:0]
            0b00 => 9,   // 4096 Hz
            0b01 => 3,   // 262144 Hz
            0b10 => 5,   // 65536 Hz
            _ => 7,      // 16384 Hz
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

    // Avança o estado do temporizador pelo número de ciclos da CPU informado
    pub fn tick_timers(&mut self, cycles: u32) {
        // Trata recarga atrasada, se ativa; processa ciclo a ciclo para manter a precisão
        for _ in 0..cycles {
            // incrementa o divisor interno a cada ciclo da CPU
            self.div_counter = self.div_counter.wrapping_add(1);
            // reflete o registrador DIV como o byte alto de div_counter
            self.memory[0xFF04] = (self.div_counter >> 8) as u8;

            // Processa recarga atrasada do TIMA
            if self.tima_reload_delay > 0 {
                self.tima_reload_delay -= 1;
                if self.tima_reload_delay == 0 {
                    // Ao fim do atraso, carrega TMA em TIMA e solicita interrupção de Timer (IF bit 2)
                    let tma = self.memory[0xFF06];
                    self.memory[0xFF05] = tma;
                    self.memory[0xFF0F] |= 0x04; // IF Timer
                    println!("[TIMER] IF(TIMER)=1; TIMA<=TMA({:02X})", tma);
                }
            }

            // Lógica do timer: incrementa TIMA na borda de descida do bit selecionado quando habilitado
            let signal = self.current_timer_signal();
            if self.timer_last_signal && !signal {
                // borda de descida
                let tima = self.memory[0xFF05];
                if tima == 0xFF {
                    // overflow: zera e inicia atraso para recarga
                    self.memory[0xFF05] = 0x00;
                    // Segundo o hardware, a recarga ocorre após 4 ciclos
                    self.tima_reload_delay = 4;
                } else {
                    self.memory[0xFF05] = tima.wrapping_add(1);
                }
            }
            self.timer_last_signal = signal;
        }
    }

    // Utilidades MBC/ROM
    fn is_mbc3(&self) -> bool {
        matches!(self.cart_type, 0x0F..=0x13)
    }

    fn rom_get(&self, idx: usize) -> u8 {
        if idx < self.rom.len() { self.rom[idx] } else { 0xFF }
    }
}